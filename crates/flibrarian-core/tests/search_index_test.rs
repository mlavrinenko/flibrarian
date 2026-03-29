use duckdb::{AccessMode, Connection};
use flibrarian_core::common::db_config;
use flibrarian_core::indexing::{check_index_state, list_archives};
use flibrarian_core::searching::{get_book_count, get_languages};
use std::path::Path;

fn init_db(db_path: &Path) -> Connection {
    let conn =
        Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite).unwrap()).unwrap();
    conn.execute_batch(include_str!("../src/schema.sql"))
        .unwrap();
    conn
}

#[test]
fn get_book_count_empty_library() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);
    drop(db_path);

    let count = get_book_count(lib_path).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn get_book_count_with_books() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('test.zip', 'indexed')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id) VALUES (1, 'Book 1', 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id) VALUES (2, 'Book 2', 1)",
        [],
    )
    .unwrap();
    drop(conn);

    let count = get_book_count(lib_path).unwrap();
    assert_eq!(count, 2);
}

#[test]
fn get_languages_empty_library() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);
    drop(db_path);

    let languages = get_languages(lib_path).unwrap();
    assert!(languages.is_empty());
}

#[test]
fn get_languages_with_books() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('test.zip', 'indexed')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id, lang) VALUES (1, 'Book 1', 1, 'en')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id, lang) VALUES (2, 'Book 2', 1, 'en')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id, lang) VALUES (3, 'Book 3', 1, 'ru')",
        [],
    )
    .unwrap();
    drop(conn);

    let languages = get_languages(lib_path).unwrap();
    assert_eq!(languages.len(), 2);
    let en = languages.iter().find(|l| l.lang == "en").unwrap();
    assert_eq!(en.count, 2);
    let ru = languages.iter().find(|l| l.lang == "ru").unwrap();
    assert_eq!(ru.count, 1);
}

#[test]
fn get_languages_ignores_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('test.zip', 'indexed')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id, lang) VALUES (1, 'Book 1', 1, 'en')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id, lang) VALUES (2, 'Book 2', 1, '')",
        [],
    )
    .unwrap();
    drop(conn);

    let languages = get_languages(lib_path).unwrap();
    assert_eq!(languages.len(), 1);
    assert_eq!(languages[0].lang, "en");
}

#[test]
fn check_index_state_no_db() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();

    let state = check_index_state(lib_path).unwrap();
    assert_eq!(state.archives_indexed, 0);
    assert_eq!(state.archives_pending, 0);
    assert!(state.search_index_valid);
    assert_eq!(state.total_books, 0);
}

#[test]
fn check_index_state_with_data() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('test.zip', 'indexed')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, archive_id) VALUES (1, 'Book 1', 1)",
        [],
    )
    .unwrap();
    drop(conn);

    let state = check_index_state(lib_path).unwrap();
    assert_eq!(state.archives_indexed, 1);
    assert_eq!(state.total_books, 1);
}

#[test]
fn check_index_state_pending_archives() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('test.zip', 'indexing')",
        [],
    )
    .unwrap();
    drop(conn);

    let state = check_index_state(lib_path).unwrap();
    assert_eq!(state.archives_pending, 1);
    assert!(state.needs_resume());
}

#[test]
fn check_index_state_needs_resume_when_new_archives_on_disk() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);

    std::fs::write(lib_path.join("unindexed.zip"), b"dummy").unwrap();

    let state = check_index_state(lib_path).unwrap();
    assert_eq!(state.archives_new, 1);
    assert!(state.needs_resume());
}

#[test]
fn check_index_state_no_resume_when_all_indexed() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('test.zip', 'indexed')",
        [],
    )
    .unwrap();
    drop(conn);

    std::fs::write(lib_path.join("test.zip"), b"dummy").unwrap();

    let state = check_index_state(lib_path).unwrap();
    assert_eq!(state.archives_indexed, 1);
    assert_eq!(state.archives_pending, 0);
    assert_eq!(state.archives_new, 0);
    assert!(!state.needs_resume());
}

#[test]
fn list_archives_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);
    drop(db_path);

    let archives = list_archives(lib_path).unwrap();
    assert!(archives.is_empty());
}

#[test]
fn list_archives_from_db() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('books.zip', 'indexed')",
        [],
    )
    .unwrap();
    drop(conn);

    let archives = list_archives(lib_path).unwrap();
    assert_eq!(archives.len(), 1);
    assert_eq!(archives[0].name, "books.zip");
    assert_eq!(archives[0].status, "indexed");
}

#[test]
fn list_archives_mixed_db_and_disk() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('existing.zip', 'indexed')",
        [],
    )
    .unwrap();
    drop(conn);

    std::fs::write(lib_path.join("new.zip"), b"dummy").unwrap();

    let archives = list_archives(lib_path).unwrap();
    assert_eq!(archives.len(), 2);
    let names: Vec<_> = archives.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"existing.zip"));
    assert!(names.contains(&"new.zip"));
}

#[test]
fn list_archives_new_on_disk() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);

    std::fs::write(lib_path.join("archive.zip"), b"dummy").unwrap();

    let archives = list_archives(lib_path).unwrap();
    assert_eq!(archives.len(), 1);
    assert_eq!(archives[0].name, "archive.zip");
    assert_eq!(archives[0].status, "new");
}

#[test]
fn list_archives_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('zebra.zip', 'indexed')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO archives (name, status) VALUES ('alpha.zip', 'indexed')",
        [],
    )
    .unwrap();
    drop(conn);

    std::fs::write(lib_path.join("middle.zip"), b"dummy").unwrap();

    let archives = list_archives(lib_path).unwrap();
    assert_eq!(archives.len(), 3);
    assert_eq!(archives[0].name, "alpha.zip");
    assert_eq!(archives[1].name, "middle.zip");
    assert_eq!(archives[2].name, "zebra.zip");
}
