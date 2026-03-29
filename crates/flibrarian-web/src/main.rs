mod handlers;

use axum::Router;
use axum::http::{StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use clap::Parser;
use rust_embed::Embed;
use std::path::PathBuf;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Embed)]
#[folder = "../../frontend/dist/"]
struct FrontendAssets;

#[derive(Parser)]
#[command(name = "flibrarian-web", about = "Web server for flibrarian")]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    #[arg(long)]
    frontend_dir: Option<PathBuf>,
}

async fn embedded_assets(uri: axum::http::Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    FrontendAssets::get(path).map_or_else(
        || {
            FrontendAssets::get("index.html").map_or_else(
                || StatusCode::NOT_FOUND.into_response(),
                |content| serve_embedded(content, "text/html"),
            )
        },
        |content| {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            serve_embedded(content, mime.as_ref())
        },
    )
}

fn serve_embedded(file: rust_embed::EmbeddedFile, content_type: &str) -> axum::response::Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type.to_string())],
        file.data.into_owned(),
    )
        .into_response()
}

fn api_routes() -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/search", get(handlers::search))
        .route("/book-count", get(handlers::book_count))
        .route("/languages", get(handlers::languages))
        .route("/index", post(handlers::index))
        .route("/index-state", get(handlers::index_state))
        .route("/archives", get(handlers::list_archives))
        .route("/extract", post(handlers::extract))
        .route("/download", post(handlers::download))
        .route("/cover", get(handlers::cover))
        .route("/annotation", get(handlers::annotation))
        .route(
            "/settings",
            get(handlers::get_settings).put(handlers::put_settings),
        )
}

fn app_with_disk_frontend(frontend_dir: &PathBuf) -> Router {
    let index_file = frontend_dir.join("index.html");

    Router::new()
        .nest("/api", api_routes())
        .fallback_service(
            ServeDir::new(frontend_dir).not_found_service(ServeFile::new(&index_file)),
        )
        .layer(CorsLayer::permissive())
}

fn app_with_embedded_frontend() -> Router {
    Router::new()
        .nest("/api", api_routes())
        .fallback(embedded_assets)
        .layer(CorsLayer::permissive())
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Args::parse();

    let app = args.frontend_dir.as_ref().map_or_else(
        || {
            log::info!("Serving embedded frontend assets");
            app_with_embedded_frontend()
        },
        |dir| {
            log::info!("Serving frontend from disk: {}", dir.display());
            app_with_disk_frontend(dir)
        },
    );

    let bind_addr = format!("{}:{}", args.host, args.port);
    log::info!("Starting server at http://{bind_addr}");

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app).await.expect("Server failed");
}
