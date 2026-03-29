use anyhow::Result;
use duckdb::{AccessMode, Connection};
use flibrarian_core::common::db_config;
use flibrarian_core::indexing::{self, IndexingMode, check_index_state};
use flibrarian_core::searching::{SearchFilters, search_library};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicBool;
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

fn index_full(lib_path: &Path) -> Result<()> {
    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &AtomicBool::new(false),
    )
}

#[test]
fn index_new_mode_skips_already_indexed() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "first.zip",
        &[(1, "First Book", "Alice", "Smith")],
    );
    index_full(lib_path)?;

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 1);
    assert_eq!(state.total_books, 1);

    create_test_zip(
        lib_path,
        "second.zip",
        &[(2, "Second Book", "Bob", "Jones")],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::New,
        |_, _, _| {},
        |_| {},
        |_| {},
        &AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 2);
    assert_eq!(state.total_books, 2);

    Ok(())
}

#[test]
fn index_archives_mode_reindexes_selected_only() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "alpha.zip",
        &[(1, "Alpha Book", "Alice", "Smith")],
    );
    create_test_zip(lib_path, "beta.zip", &[(2, "Beta Book", "Bob", "Jones")]);
    create_test_zip(
        lib_path,
        "gamma.zip",
        &[(3, "Gamma Book", "Carol", "White")],
    );

    index_full(lib_path)?;
    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 3);

    create_test_zip(
        lib_path,
        "alpha.zip",
        &[(1, "Alpha Updated", "Alice", "Smith")],
    );

    indexing::index_library(
        lib_path,
        &IndexingMode::Archives(vec!["alpha.zip".to_string()]),
        |_, _, _| {},
        |_| {},
        |_| {},
        &AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 3);
    assert_eq!(state.total_books, 3);

    let results = search_library(lib_path, "Alpha Updated", &SearchFilters::default())?;
    assert_eq!(results.len(), 1);

    Ok(())
}

#[test]
fn index_search_mode_rebuilds_search_index() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "books.zip",
        &[(1, "Searchable Book", "Test", "Author")],
    );
    index_full(lib_path)?;

    let db_path = lib_path.join("lib.duckdb");
    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite)?)?;
    conn.execute_batch("DROP TABLE IF EXISTS search_index")?;
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert!(!state.search_index_valid);

    indexing::index_library(
        lib_path,
        &IndexingMode::Search,
        |_, _, _| {},
        |_| {},
        |_| {},
        &AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert!(state.search_index_valid);

    let results = search_library(lib_path, "Searchable", &SearchFilters::default())?;
    assert_eq!(results.len(), 1);

    Ok(())
}

#[test]
fn index_cancelled_stops_early() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(lib_path, "books.zip", &[(1, "Book One", "A", "B")]);

    let cancelled = AtomicBool::new(true);
    let result = indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &cancelled,
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("cancelled"),
        "Expected cancellation error, got: {err_msg}"
    );

    Ok(())
}

#[test]
fn index_full_mode_reindexes_existing() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    create_test_zip(
        lib_path,
        "books.zip",
        &[(1, "Original Title", "Test", "Author")],
    );
    index_full(lib_path)?;

    create_test_zip(
        lib_path,
        "books.zip",
        &[(1, "Updated Title", "Test", "Author")],
    );
    index_full(lib_path)?;

    let results = search_library(lib_path, "Updated", &SearchFilters::default())?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Updated Title");

    Ok(())
}
