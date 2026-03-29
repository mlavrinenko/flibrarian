use base64::Engine;
use flibrarian_core::common::resolve_path;
use flibrarian_core::extracting::ExtractedBook;
use flibrarian_core::indexing::{
    ArchiveInfo, IndexState, IndexingInfo, IndexingMode, IndexingProgress, IndexingWarning,
};
use flibrarian_core::searching::{FoundBook, LanguageCount, SearchFilters};
use flibrarian_core::settings::Settings;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Emitter;

pub struct IndexingState {
    pub cancelled: Mutex<Option<Arc<AtomicBool>>>,
}

#[tauri::command]
pub async fn get_languages(path: String) -> Result<Vec<LanguageCount>, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::searching::get_languages(&library_path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_book_count(path: String) -> Result<u64, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::searching::get_book_count(&library_path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn check_index_state(path: String) -> Result<IndexState, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::indexing::check_index_state(&library_path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_archives(path: String) -> Result<Vec<ArchiveInfo>, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::indexing::list_archives(&library_path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_library(
    path: String,
    query: String,
    filters: SearchFilters,
) -> Result<Vec<FoundBook>, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::searching::search_library(&library_path, &query, &filters)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn index_library(
    app: tauri::AppHandle,
    state: tauri::State<'_, IndexingState>,
    path: String,
    mode: String,
    archives: Option<Vec<String>>,
) -> Result<(), String> {
    let library_path = resolve_path(&path);
    let indexing_mode = IndexingMode::from_str_with_archives(&mode, archives);
    let cancelled = Arc::new(AtomicBool::new(false));
    *state.cancelled.lock().unwrap() = Some(Arc::clone(&cancelled));

    let result = tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::indexing::index_library(
            &library_path,
            &indexing_mode,
            |phase, current, total| {
                let _ = app.emit(
                    "indexing-progress",
                    IndexingProgress {
                        phase,
                        current,
                        total,
                    },
                );
            },
            |message| {
                let _ = app.emit(
                    "indexing-warning",
                    IndexingWarning {
                        message: message.to_string(),
                    },
                );
            },
            |message| {
                let _ = app.emit(
                    "indexing-info",
                    IndexingInfo {
                        message: message.to_string(),
                    },
                );
            },
            &cancelled,
        )
    })
    .await
    .map_err(|e| e.to_string())?;

    *state.cancelled.lock().unwrap() = None;
    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cancel_indexing(state: tauri::State<'_, IndexingState>) -> Result<(), String> {
    if let Some(flag) = state.cancelled.lock().unwrap().as_ref() {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
pub async fn extract_books(
    path: String,
    book_ids: Vec<u32>,
    output_dir: String,
) -> Result<Vec<ExtractedBook>, String> {
    let library_path = resolve_path(&path);
    let output_path = resolve_path(&output_dir);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::extracting::extract_books(&library_path, &book_ids, &output_path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    tauri::async_runtime::spawn_blocking(flibrarian_core::settings::load_settings)
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::settings::save_settings(&settings)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[derive(Clone, Serialize)]
pub struct CoverData {
    pub data: String,
    pub content_type: String,
}

#[tauri::command]
pub async fn get_book_cover(path: String, book_id: u32) -> Result<Option<CoverData>, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::covers::get_book_cover(&library_path, book_id)
    })
    .await
    .map_err(|e| e.to_string())?
    .map(|opt| {
        opt.map(|cover| CoverData {
            data: base64::engine::general_purpose::STANDARD.encode(&cover.data),
            content_type: cover.content_type,
        })
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_book_annotation(path: String, book_id: u32) -> Result<Option<String>, String> {
    let library_path = resolve_path(&path);
    tauri::async_runtime::spawn_blocking(move || {
        flibrarian_core::annotations::get_book_annotation(&library_path, book_id)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}
