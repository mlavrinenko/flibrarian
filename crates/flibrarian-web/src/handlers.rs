use axum::Json;
use axum::body::Body;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use flibrarian_core::common::resolve_path;
use flibrarian_core::extracting::ExtractedBook;
use flibrarian_core::indexing::{ArchiveInfo, IndexState, IndexingMode, IndexingProgress};
use flibrarian_core::searching::{FoundBook, LanguageCount, SearchFilters};
use flibrarian_core::settings::Settings;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::io::{Cursor, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Deserialize)]
pub struct SearchParams {
    path: String,
    #[serde(default)]
    query: String,
    filter_title: Option<String>,
    filter_authors: Option<String>,
    filter_genres: Option<String>,
    filter_date: Option<String>,
    filter_lang: Option<String>,
    filter_file_size: Option<String>,
    filter_sequence: Option<String>,
}

#[derive(Deserialize)]
pub struct IndexRequest {
    path: String,
    mode: String,
    archives: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct ExtractRequest {
    path: String,
    book_ids: Vec<u32>,
    output_dir: String,
}

#[derive(Deserialize)]
pub struct DownloadRequest {
    path: String,
    book_ids: Vec<u32>,
}

#[derive(Deserialize)]
pub struct CoverParams {
    path: String,
    book_id: u32,
}

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn error_response(status: StatusCode, message: String) -> impl IntoResponse {
    log::error!("{status} {message}");
    (status, Json(ErrorBody { error: message }))
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}

#[derive(Deserialize)]
pub struct BookCountParams {
    path: String,
}

pub async fn languages(
    Query(params): Query<BookCountParams>,
) -> Result<Json<Vec<LanguageCount>>, impl IntoResponse> {
    let library_path = resolve_path(&params.path);
    tokio::task::spawn_blocking(move || flibrarian_core::searching::get_languages(&library_path))
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn book_count(
    Query(params): Query<BookCountParams>,
) -> Result<Json<u64>, impl IntoResponse> {
    let library_path = resolve_path(&params.path);
    tokio::task::spawn_blocking(move || flibrarian_core::searching::get_book_count(&library_path))
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn index_state(
    Query(params): Query<BookCountParams>,
) -> Result<Json<IndexState>, impl IntoResponse> {
    let library_path = resolve_path(&params.path);
    tokio::task::spawn_blocking(move || flibrarian_core::indexing::check_index_state(&library_path))
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn search(
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<FoundBook>>, impl IntoResponse> {
    let library_path = resolve_path(&params.path);
    let query = params.query;
    let filters = SearchFilters {
        title: params.filter_title,
        authors: params.filter_authors,
        genres: params.filter_genres,
        date: params.filter_date,
        lang: params.filter_lang,
        file_size: params.filter_file_size,
        sequence: params.filter_sequence,
    };

    tokio::task::spawn_blocking(move || {
        flibrarian_core::searching::search_library(&library_path, &query, &filters)
    })
    .await
    .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn index(
    Json(request): Json<IndexRequest>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, Response> {
    let library_path = resolve_path(&request.path);

    if let Err(e) = flibrarian_core::preflight::ensure_writable(&library_path) {
        return Err(error_response(StatusCode::FORBIDDEN, format!("{e:#}")).into_response());
    }

    let indexing_mode = IndexingMode::from_str_with_archives(&request.mode, request.archives);

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);
    let cancelled = Arc::new(AtomicBool::new(false));

    tokio::task::spawn_blocking({
        let cancelled = Arc::clone(&cancelled);
        move || {
            let cancelled_for_send = Arc::clone(&cancelled);
            let result = flibrarian_core::indexing::index_library(
                &library_path,
                &indexing_mode,
                |phase, current, total| {
                    let progress = IndexingProgress {
                        phase,
                        current,
                        total,
                    };
                    if let Ok(event) = Event::default().json_data(&progress)
                        && tx.blocking_send(Ok(event)).is_err()
                    {
                        cancelled_for_send.store(true, Ordering::Relaxed);
                    }
                },
                |message| {
                    let event = Event::default().event("warning").data(message);
                    let _ = tx.blocking_send(Ok(event));
                },
                |message| {
                    let event = Event::default().event("info").data(message);
                    let _ = tx.blocking_send(Ok(event));
                },
                &cancelled,
            );

            match result {
                Ok(()) => {
                    let event = Event::default().event("done").data("Indexing complete");
                    let _ = tx.blocking_send(Ok(event));
                }
                Err(e) => {
                    let event = Event::default().event("error").data(e.to_string());
                    let _ = tx.blocking_send(Ok(event));
                }
            }
        }
    });

    Ok(Sse::new(ReceiverStream::new(rx)))
}

pub async fn extract(
    Json(request): Json<ExtractRequest>,
) -> Result<Json<Vec<ExtractedBook>>, impl IntoResponse> {
    let library_path = resolve_path(&request.path);
    let book_ids = request.book_ids;
    let output_dir = resolve_path(&request.output_dir);

    tokio::task::spawn_blocking(move || {
        flibrarian_core::extracting::extract_books(&library_path, &book_ids, &output_dir)
    })
    .await
    .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_settings() -> Result<Json<Settings>, impl IntoResponse> {
    tokio::task::spawn_blocking(flibrarian_core::settings::load_settings)
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn put_settings(Json(settings): Json<Settings>) -> Result<StatusCode, impl IntoResponse> {
    tokio::task::spawn_blocking(move || flibrarian_core::settings::save_settings(&settings))
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(|()| StatusCode::NO_CONTENT)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn cover(Query(params): Query<CoverParams>) -> Response {
    let library_path = resolve_path(&params.path);
    let book_id = params.book_id;

    let result = tokio::task::spawn_blocking(move || {
        flibrarian_core::covers::get_book_cover(&library_path, book_id)
    })
    .await;

    let cover = match result {
        Ok(Ok(Some(c))) => c,
        Ok(Ok(None)) => return StatusCode::NOT_FOUND.into_response(),
        Ok(Err(e)) => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                .into_response();
        }
        Err(e) => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                .into_response();
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", cover.content_type)
        .body(Body::from(cover.data))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn annotation(
    Query(params): Query<CoverParams>,
) -> Result<Json<Option<String>>, impl IntoResponse> {
    let library_path = resolve_path(&params.path);
    let book_id = params.book_id;

    tokio::task::spawn_blocking(move || {
        flibrarian_core::annotations::get_book_annotation(&library_path, book_id)
    })
    .await
    .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .map(Json)
    .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn list_archives(
    Query(params): Query<BookCountParams>,
) -> Result<Json<Vec<ArchiveInfo>>, impl IntoResponse> {
    let library_path = resolve_path(&params.path);
    tokio::task::spawn_blocking(move || flibrarian_core::indexing::list_archives(&library_path))
        .await
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .map_err(|e| error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

fn encode_content_disposition(filename: &str) -> String {
    let encoded: String = filename
        .bytes()
        .flat_map(|b| {
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' {
                vec![b as char]
            } else {
                format!("%{b:02X}").chars().collect()
            }
        })
        .collect();
    format!("attachment; filename*=UTF-8''{encoded}")
}

pub async fn download(Json(request): Json<DownloadRequest>) -> Response {
    let library_path = resolve_path(&request.path);
    let book_ids = request.book_ids;

    let result = tokio::task::spawn_blocking(move || {
        flibrarian_core::extracting::extract_book_contents(&library_path, &book_ids)
    })
    .await;

    let books = match result {
        Ok(Ok(b)) => b,
        Ok(Err(e)) => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                .into_response();
        }
        Err(e) => {
            return error_response(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                .into_response();
        }
    };

    if books.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if books.len() == 1 {
        let book = &books[0];
        return Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/xml")
            .header(
                CONTENT_DISPOSITION,
                encode_content_disposition(&book.file_name),
            )
            .body(Body::from(book.data.clone()))
            .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    }

    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip_writer = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for book in &books {
            if zip_writer.start_file(&book.file_name, options).is_ok() {
                let _ = zip_writer.write_all(&book.data);
            }
        }
        let _ = zip_writer.finish();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/zip")
        .header(CONTENT_DISPOSITION, "attachment; filename=\"books.zip\"")
        .body(Body::from(buf.into_inner()))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}
