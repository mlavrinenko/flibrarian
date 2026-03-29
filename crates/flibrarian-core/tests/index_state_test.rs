use anyhow::Result;
use duckdb::{AccessMode, Connection};
use flibrarian_core::common::db_config;
use flibrarian_core::indexing::{self, IndexingMode, check_index_state};
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

#[test]
fn check_index_state_empty_library() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 0);
    assert_eq!(state.archives_pending, 0);
    assert_eq!(state.archives_new, 0);
    assert!(state.search_index_valid);
    assert_eq!(state.total_books, 0);
    assert!(!state.needs_resume());

    Ok(())
}

#[test]
fn check_index_state_zips_but_no_db() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "archive1.zip",
        &[(1, "Book One", "Alice", "Smith")],
    );
    create_test_zip(lib_path, "archive2.zip", &[(2, "Book Two", "Bob", "Jones")]);

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_new, 2);
    assert!(state.needs_resume());

    Ok(())
}

#[test]
fn old_search_index_survives_if_new_build_incomplete() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive.zip",
        &[(1, "Original Book", "Test", "Author")],
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

    let count_before: usize =
        conn.query_row("SELECT COUNT(*) FROM search_index", [], |row| row.get(0))?;
    assert_eq!(count_before, 1);

    conn.execute_batch(
        "CREATE TABLE search_index_new (id UINTEGER, title VARCHAR, genres_text VARCHAR, authors_text VARCHAR, sequence VARCHAR)",
    )?;
    drop(conn);

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    let count_after: usize =
        conn.query_row("SELECT COUNT(*) FROM search_index", [], |row| row.get(0))?;
    assert_eq!(count_after, 1);

    let results = search_library(lib_path, "Original", &SearchFilters::default())?;
    assert_eq!(results.len(), 1);

    Ok(())
}

#[test]
fn check_index_state_detects_missing_search_index() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    create_test_zip(
        lib_path,
        "archive.zip",
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
    assert!(state.search_index_valid);

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite)?)?;
    conn.execute_batch("DROP TABLE IF EXISTS search_index")?;
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert!(!state.search_index_valid);

    Ok(())
}

#[test]
fn fully_indexed_library_reports_clean_state() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "archive1.zip",
        &[(1, "Book One", "Alice", "Smith")],
    );
    create_test_zip(lib_path, "archive2.zip", &[(2, "Book Two", "Bob", "Jones")]);

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 2);
    assert_eq!(state.archives_pending, 0);
    assert_eq!(state.archives_new, 0);
    assert!(state.search_index_valid);
    assert_eq!(state.total_books, 2);
    assert!(!state.needs_resume());

    Ok(())
}
