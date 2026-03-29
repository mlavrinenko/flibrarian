use anyhow::Result;
use duckdb::{AccessMode, Connection, params};
use flibrarian_core::common::db_config;
use flibrarian_core::extracting::extract_books;
use flibrarian_core::indexing::{self, IndexingMode, IndexingPhase, create_search_index};
use flibrarian_core::searching::{SearchFilters, search_library};
use std::fs;
use std::io::Write;
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn minimal_fb2(id: u32, title: &str, author_first: &str, author_last: &str) -> Vec<u8> {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<genre>fantasy</genre>
<author><first-name>{author_first}</first-name><last-name>{author_last}</last-name></author>
<book-title>{title}</book-title>
<date>2023</date>
<sequence name="TestSeries" number="1"/>
</title-info>
</description>
<body><section><p>Content of book {id}</p></section></body>
</FictionBook>"#
    )
    .into_bytes()
}

fn create_test_zip(dir: &Path, zip_name: &str, books: &[(u32, &str, &str, &str)]) {
    let zip_path = dir.join(zip_name);
    let file = fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    for (id, title, first, last) in books {
        let fb2_data = minimal_fb2(*id, title, first, last);
        zip.start_file(format!("{id}.fb2"), options).unwrap();
        zip.write_all(&fb2_data).unwrap();
    }

    zip.finish().unwrap();
}

fn init_db(db_path: &Path) -> Connection {
    let conn =
        Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite).unwrap()).unwrap();
    conn.execute_batch(include_str!("../src/schema.sql"))
        .unwrap();
    conn
}

fn populate_db(conn: &Connection, archive_name: &str, books: &[(u32, &str, &str, &str)]) {
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

    for (id, title, first, last) in books {
        conn.execute(
            "INSERT OR REPLACE INTO books (id, title, genres, date, sequence, archive_id) VALUES (?, ?, '[\"fantasy\"]', '2023', 'TestSeries 1', ?)",
            params![id, title, archive_id],
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

// --- extract_books tests ---

#[test]
fn test_extract_books_from_zip() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "archive.zip",
        &[
            (1, "Extract Me", "Test", "Author"),
            (2, "Extract Me Too", "Another", "Writer"),
        ],
    );
    drop(conn);

    create_test_zip(
        lib_path,
        "archive.zip",
        &[
            (1, "Extract Me", "Test", "Author"),
            (2, "Extract Me Too", "Another", "Writer"),
        ],
    );

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[1, 2], output_dir.path())?;

    let extracted_count = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fb2"))
        .count();

    assert_eq!(extracted_count, 2);

    Ok(())
}

#[test]
fn test_extract_books_nonexistent_ids() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(&conn, "archive.zip", &[(1, "Only Book", "Test", "Author")]);
    drop(conn);

    create_test_zip(
        lib_path,
        "archive.zip",
        &[(1, "Only Book", "Test", "Author")],
    );

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[999, 998], output_dir.path())?;

    let extracted_count = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fb2"))
        .count();

    assert_eq!(extracted_count, 0);

    Ok(())
}

#[test]
fn test_extract_single_book_filename_format() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "archive.zip",
        &[(7, "My Great Novel", "Jane", "Austen")],
    );
    drop(conn);

    create_test_zip(
        lib_path,
        "archive.zip",
        &[(7, "My Great Novel", "Jane", "Austen")],
    );

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[7], output_dir.path())?;

    let entries: Vec<_> = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .collect();

    assert_eq!(entries.len(), 1);
    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(filename.contains('7'));
    assert!(
        Path::new(&filename)
            .extension()
            .is_some_and(|ext| ext == "fb2")
    );

    Ok(())
}

#[test]
fn test_extract_from_missing_archive_file() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "missing.zip",
        &[(1, "Ghost Book", "Ghost", "Author")],
    );
    drop(conn);

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[1], output_dir.path())?;

    let extracted_count = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fb2"))
        .count();

    assert_eq!(extracted_count, 0);

    Ok(())
}

#[test]
fn test_extract_book_not_in_archive() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    populate_db(
        &conn,
        "archive.zip",
        &[(99, "Missing Inside", "Test", "Author")],
    );
    drop(conn);

    create_test_zip(
        lib_path,
        "archive.zip",
        &[(1, "Different Book", "Other", "Author")],
    );

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[99], output_dir.path())?;

    let extracted_count = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "fb2"))
        .count();

    assert_eq!(extracted_count, 0);

    Ok(())
}
