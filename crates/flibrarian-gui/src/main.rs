#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use std::sync::Mutex;

use commands::{
    IndexingState, cancel_indexing, check_index_state, extract_books, get_book_annotation,
    get_book_count, get_book_cover, get_languages, get_settings, index_library, list_archives,
    save_settings, search_library,
};

fn main() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(IndexingState {
            cancelled: Mutex::default(),
        })
        .invoke_handler(tauri::generate_handler![
            search_library,
            index_library,
            cancel_indexing,
            check_index_state,
            list_archives,
            extract_books,
            get_book_count,
            get_languages,
            get_book_cover,
            get_book_annotation,
            get_settings,
            save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run tauri application");
}
