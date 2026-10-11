use anyhow::Result;
use duckdb::{AccessMode, Connection};
use flibrarian_core::common::db_config;
use flibrarian_core::indexing::{self, IndexingMode, IndexingPhase, create_search_index};
use flibrarian_core::searching::{SearchFilters, search_library};

use crate::common::{create_test_zip, init_db, populate_db};

#[test]
fn test_index_library_indexes_books() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(lib_path, "test.zip", &[(1, "Book", "A", "B")]);

    let result = indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    );
    assert!(result.is_ok(), "index_library returned error: {result:?}");

    let conn =
        Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite).unwrap()).unwrap();
    let book_count: u32 = conn
        .query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
        .unwrap();
    assert!(book_count > 0, "Expected books to be indexed, got 0");
}

#[test]
fn test_search_library_creates_index_on_demand() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(&conn, "test.zip", &[(1, "Test Book", "John", "Doe")]);
    drop(conn);

    let result = search_library(lib_path, "Test", &SearchFilters::default());
    assert!(
        result.is_ok(),
        "search_library should create index on demand: {result:?}"
    );
}

// --- create_search_index tests ---

#[test]
fn test_create_search_index() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let db_path = tmp.path().join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "test.zip",
        &[(1, "Searchable Book", "Test", "Author")],
    );

    create_search_index(
        &conn,
        &|_: IndexingPhase, _, _| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let has_index: bool = conn
        .prepare(
            "SELECT table_name FROM information_schema.tables WHERE table_name='search_index'",
        )?
        .query_row([], |row| row.get::<usize, String>(0))
        .is_ok();

    assert!(has_index);

    Ok(())
}

#[test]
fn test_create_search_index_aggregates_data() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let db_path = tmp.path().join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "archive.zip",
        &[
            (1, "Book One", "Alice", "Smith"),
            (2, "Book Two", "Bob", "Jones"),
        ],
    );

    create_search_index(
        &conn,
        &|_: IndexingPhase, _, _| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let count: u32 = conn.query_row("SELECT COUNT(*) FROM search_index", [], |row| row.get(0))?;
    assert_eq!(count, 2);

    Ok(())
}

// --- search_library tests (with pre-built index) ---

#[test]
fn test_search_library_with_results() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "books.zip",
        &[
            (1, "Adventures in Wonderland", "Lewis", "Carroll"),
            (2, "War and Peace", "Leo", "Tolstoy"),
            (3, "Crime and Punishment", "Fyodor", "Dostoevsky"),
        ],
    );

    create_search_index(
        &conn,
        &|_: IndexingPhase, _, _| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;
    drop(conn);

    search_library(lib_path, "Adventures", &SearchFilters::default())?;
    search_library(lib_path, "Tolstoy", &SearchFilters::default())?;

    Ok(())
}

#[test]
fn test_search_library_no_results() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(&conn, "books.zip", &[(1, "Some Book", "Some", "Author")]);

    create_search_index(
        &conn,
        &|_: IndexingPhase, _, _| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;
    drop(conn);

    search_library(lib_path, "xyznonexistent", &SearchFilters::default())?;

    Ok(())
}
