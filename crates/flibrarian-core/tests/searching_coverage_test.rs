use anyhow::Result;
use duckdb::{AccessMode, Connection, params};
use flibrarian_core::common::db_config;
use flibrarian_core::indexing::IndexingPhase;
use flibrarian_core::searching::{SearchFilters, get_book_count, get_languages, search_library};
use std::path::Path;
use std::sync::atomic::AtomicBool;

fn init_db(db_path: &Path) -> Connection {
    let conn =
        Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite).unwrap()).unwrap();
    conn.execute_batch(include_str!("../src/schema.sql"))
        .unwrap();
    conn
}

fn populate_db(conn: &Connection, archive_name: &str, books: &[(u32, &str, &str, &str, &str)]) {
    let archive_id: u32 = conn
        .query_row(
            "SELECT id FROM archives WHERE name = ?",
            params![archive_name],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| {
            conn.execute(
                "INSERT INTO archives (name, status) VALUES (?, 'indexed')",
                params![archive_name],
            )
            .unwrap();
            conn.query_row(
                "SELECT id FROM archives WHERE name = ?",
                params![archive_name],
                |row| row.get(0),
            )
            .unwrap()
        });

    for (id, title, first, last, lang) in books {
        conn.execute(
            "INSERT OR REPLACE INTO books (id, title, genres, date, lang, file_size, sequence, archive_id) VALUES (?, ?, '[\"fantasy\"]', '2023', ?, 1000, 'TestSeries 1', ?)",
            params![id, title, lang, archive_id],
        )
        .unwrap();

        let author_id = format!("author-{id}");
        conn.execute(
            "INSERT OR IGNORE INTO authors (id, first_name, middle_name, last_name, nickname) VALUES (?, ?, NULL, ?, NULL)",
            params![&author_id, first, last],
        )
        .unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO books_authors (book_id, author_id) VALUES (?, ?)",
            params![id, &author_id],
        )
        .unwrap();
    }
}

#[test]
fn get_book_count_no_db_returns_zero() {
    let tmp = tempfile::tempdir().unwrap();
    let count = get_book_count(tmp.path()).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_languages_no_db_returns_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let languages = get_languages(tmp.path()).unwrap();
    assert!(languages.is_empty());
}

#[test]
fn search_library_empty_query_uses_filter_only() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "books.zip",
        &[
            (1, "War and Peace", "Leo", "Tolstoy", "ru"),
            (2, "Crime and Punishment", "Fyodor", "Dostoevsky", "ru"),
            (3, "Pride and Prejudice", "Jane", "Austen", "en"),
        ],
    );
    drop(conn);

    let results = search_library(lib_path, "", &SearchFilters::default())?;
    assert_eq!(results.len(), 3);

    Ok(())
}

#[test]
fn search_library_empty_query_with_lang_filter() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "books.zip",
        &[
            (1, "War and Peace", "Leo", "Tolstoy", "ru"),
            (2, "Crime and Punishment", "Fyodor", "Dostoevsky", "ru"),
            (3, "Pride and Prejudice", "Jane", "Austen", "en"),
        ],
    );
    drop(conn);

    let filters = SearchFilters {
        lang: Some("ru".to_string()),
        ..SearchFilters::default()
    };
    let results = search_library(lib_path, "", &filters)?;
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn search_library_fts_query_with_filter() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "books.zip",
        &[
            (1, "War and Peace", "Leo", "Tolstoy", "ru"),
            (2, "Crime and Punishment", "Fyodor", "Dostoevsky", "ru"),
            (3, "Pride and Prejudice", "Jane", "Austen", "en"),
        ],
    );

    flibrarian_core::indexing::create_search_index(
        &conn,
        &|_: IndexingPhase, _, _| {},
        &AtomicBool::new(false),
    )?;
    drop(conn);

    let filters = SearchFilters {
        lang: Some("ru".to_string()),
        ..SearchFilters::default()
    };
    let results = search_library(lib_path, "War", &filters)?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "War and Peace");

    Ok(())
}
