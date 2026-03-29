use anyhow::Result;
use duckdb::{AccessMode, Connection, params};
use flibrarian_core::common::db_config;
use flibrarian_core::indexing::{
    self, ArchiveStatus, IndexingMode, check_index_state, write_books_to_db,
};
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

fn count_books(conn: &Connection) -> usize {
    conn.query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
        .unwrap()
}

fn count_archives_by_status(conn: &Connection, status: &ArchiveStatus) -> usize {
    conn.query_row(
        "SELECT COUNT(*) FROM archives WHERE status = ?",
        params![status],
        |row| row.get(0),
    )
    .unwrap()
}

fn has_search_index(conn: &Connection) -> bool {
    conn.prepare("SELECT table_name FROM information_schema.tables WHERE table_name='search_index'")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<usize, String>(0)))
        .is_ok()
}

#[test]
fn crash_during_write_phase_preserves_committed_archives() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive1.zip",
        &[(1, "Book One", "Alice", "Smith")],
    );
    create_test_zip(lib_path, "archive2.zip", &[(2, "Book Two", "Bob", "Jones")]);
    create_test_zip(
        lib_path,
        "archive3.zip",
        &[(3, "Book Three", "Carol", "White")],
    );

    let conn = init_db(&db_path);
    drop(conn);

    indexing::parse_book_from_bytes(1, &minimal_fb2(1, "Book One", "Alice", "Smith"))?;

    let books1 = vec![indexing::parse_book_from_bytes(
        1,
        &minimal_fb2(1, "Book One", "Alice", "Smith"),
    )?];
    write_books_to_db(&db_path, "archive1.zip", &books1)?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    assert_eq!(count_archives_by_status(&conn, &ArchiveStatus::Indexed), 1);
    assert_eq!(count_books(&conn), 1);
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 1);
    assert_eq!(state.archives_new, 2);
    assert!(state.needs_resume());

    indexing::index_library(
        lib_path,
        &IndexingMode::New,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    assert_eq!(count_archives_by_status(&conn, &ArchiveStatus::Indexed), 3);
    assert_eq!(count_books(&conn), 3);
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert!(!state.needs_resume());
    assert!(state.search_index_valid);

    let results = search_library(lib_path, "Book", &SearchFilters::default())?;
    assert_eq!(results.len(), 3);

    Ok(())
}

#[test]
fn interrupted_archive_gets_retried_on_resume() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive1.zip",
        &[(1, "Book One", "Alice", "Smith")],
    );
    create_test_zip(lib_path, "archive2.zip", &[(2, "Book Two", "Bob", "Jones")]);

    let conn = init_db(&db_path);

    let books1 = vec![indexing::parse_book_from_bytes(
        1,
        &minimal_fb2(1, "Book One", "Alice", "Smith"),
    )?];
    drop(conn);
    write_books_to_db(&db_path, "archive1.zip", &books1)?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite)?)?;
    conn.execute(
        "INSERT INTO archives (name, status) VALUES (?, ?)",
        params!["archive2.zip", ArchiveStatus::Indexing],
    )?;
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 1);
    assert_eq!(state.archives_pending, 1);
    assert_eq!(state.archives_new, 0);
    assert!(state.needs_resume());

    indexing::index_library(
        lib_path,
        &IndexingMode::New,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    assert_eq!(count_archives_by_status(&conn, &ArchiveStatus::Indexed), 2);
    assert_eq!(count_archives_by_status(&conn, &ArchiveStatus::Indexing), 0);
    assert_eq!(count_books(&conn), 2);

    Ok(())
}

#[test]
fn missing_search_index_auto_rebuilds_on_search() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive.zip",
        &[
            (1, "Adventures in Wonderland", "Lewis", "Carroll"),
            (2, "War and Peace", "Leo", "Tolstoy"),
        ],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite)?)?;
    assert!(has_search_index(&conn));
    conn.execute_batch("DROP TABLE IF EXISTS search_index")?;
    assert!(!has_search_index(&conn));
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert!(!state.search_index_valid);

    let results = search_library(lib_path, "Adventures", &SearchFilters::default())?;
    assert!(!results.is_empty());

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    assert!(has_search_index(&conn));

    Ok(())
}

#[test]
fn new_archives_detected_after_full_index() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "archive1.zip",
        &[(1, "Book One", "Alice", "Smith")],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert!(!state.needs_resume());

    create_test_zip(lib_path, "archive2.zip", &[(2, "Book Two", "Bob", "Jones")]);

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_new, 1);
    assert!(state.needs_resume());

    indexing::index_library(
        lib_path,
        &IndexingMode::New,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert!(!state.needs_resume());
    assert_eq!(state.total_books, 2);

    let results = search_library(lib_path, "Book Two", &SearchFilters::default())?;
    assert!(!results.is_empty());

    Ok(())
}

#[test]
fn reindex_same_archive_is_idempotent() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive.zip",
        &[
            (1, "Book One", "Alice", "Smith"),
            (2, "Book Two", "Bob", "Jones"),
        ],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    let count_before = count_books(&conn);
    drop(conn);

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    let count_after = count_books(&conn);
    assert_eq!(count_before, count_after);

    let results = search_library(lib_path, "Book", &SearchFilters::default())?;
    assert_eq!(results.len(), 2);

    Ok(())
}

#[test]
fn search_index_includes_all_books_after_resume() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive1.zip",
        &[(1, "Unique Alpha Title", "Alice", "Smith")],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    create_test_zip(
        lib_path,
        "archive2.zip",
        &[(2, "Unique Beta Title", "Bob", "Jones")],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::New,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    let search_count: usize =
        conn.query_row("SELECT COUNT(*) FROM search_index", [], |row| row.get(0))?;
    assert_eq!(search_count, 2);
    drop(conn);

    let results = search_library(lib_path, "Alpha", &SearchFilters::default())?;
    assert_eq!(results.len(), 1);

    let results = search_library(lib_path, "Beta", &SearchFilters::default())?;
    assert_eq!(results.len(), 1);

    Ok(())
}
