#![cfg(feature = "faking")]

use anyhow::Result;
use duckdb::{AccessMode, Connection};
use flibrarian_core::common::db_config;
use flibrarian_core::faking::FakeLibraryConfig;
use flibrarian_core::indexing::{self, IndexingMode, check_index_state};
use std::fs;
use std::path::Path;

fn count_books(conn: &Connection) -> usize {
    conn.query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
        .unwrap()
}

fn generate_test_library(lib_path: &Path, num_archives: u32, books_per_archive: u32) {
    flibrarian_core::faking::generate_fake_library(
        &FakeLibraryConfig {
            output_dir: lib_path.to_path_buf(),
            num_archives,
            books_per_archive,
            seed: Some(42),
            lang: "en".to_string(),
        },
        |_, _| {},
    )
    .unwrap();
}

#[test]
fn large_dataset_full_index_and_reindex() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    generate_test_library(lib_path, 3, 500);

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    let count_first: usize = count_books(&conn);
    assert_eq!(count_first, 1500);
    drop(conn);

    let state = check_index_state(lib_path)?;
    assert!(!state.needs_resume());
    assert!(state.search_index_valid);

    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)?;
    let count_second: usize = count_books(&conn);
    assert_eq!(count_first, count_second);

    Ok(())
}

#[test]
fn large_dataset_partial_then_resume() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();

    generate_test_library(lib_path, 1, 300);
    indexing::index_library(
        lib_path,
        &IndexingMode::Full,
        |_, _, _| {},
        |_| {},
        |_| {},
        &std::sync::atomic::AtomicBool::new(false),
    )?;

    let state = check_index_state(lib_path)?;
    assert_eq!(state.archives_indexed, 1);
    assert!(!state.needs_resume());

    generate_test_library(lib_path, 4, 300);

    let zip_count = fs::read_dir(lib_path)?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "zip"))
        .count();
    assert!(zip_count >= 4);

    let state = check_index_state(lib_path)?;
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
    assert!(state.archives_indexed >= 4);
    assert!(!state.needs_resume());
    assert!(state.total_books >= 1200);

    assert!(state.search_index_valid);

    Ok(())
}
