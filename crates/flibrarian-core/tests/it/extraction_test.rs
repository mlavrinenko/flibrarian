use anyhow::Result;
use flibrarian_core::extracting::extract_books;
use std::fs;
use std::path::Path;

use crate::common::{
    add_author, create_test_zip, init_db, populate_db, populate_db_without_authors,
};

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
fn test_extract_books_truncates_long_author_list() -> Result<()> {
    const MAX_FILE_NAME_LEN: usize = 255;

    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let title = "Название книги";
    let book_id = 123_456;
    let conn = init_db(&db_path);
    populate_db(&conn, "archive.zip", &[(book_id, title, "Джек", "Джон")]);
    for i in 0..30 {
        add_author(
            &conn,
            book_id,
            &format!("extra-author-{i}"),
            "Оченьдлинноеимя",
        );
    }
    drop(conn);

    create_test_zip(lib_path, "archive.zip", &[(book_id, title, "Джек", "Джон")]);

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[book_id], output_dir.path())?;

    let entries: Vec<_> = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(
        filename.len() <= MAX_FILE_NAME_LEN,
        "filename is {} bytes: {filename}",
        filename.len()
    );
    assert!(filename.ends_with(&format!("... - {title} [{book_id}].fb2")));

    Ok(())
}

#[test]
fn test_extract_books_truncates_long_title_without_author() -> Result<()> {
    const MAX_FILE_NAME_LEN: usize = 255;

    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let title = "Очень длинное, длинное, длинное название для книги, содержащее много-много букв и разных символов. ".repeat(4);
    let book_id = 456_123;
    let conn = init_db(&db_path);
    populate_db_without_authors(&conn, "archive.zip", &[(book_id, &title)]);
    drop(conn);

    create_test_zip(lib_path, "archive.zip", &[(book_id, &title, "", "")]);

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[book_id], output_dir.path())?;

    let entries: Vec<_> = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(
        filename.len() <= MAX_FILE_NAME_LEN,
        "filename is {} bytes: {filename}",
        filename.len()
    );
    assert!(
        !filename.starts_with(" - "),
        "filename should not start with a separator: {filename}"
    );
    assert!(filename.ends_with(&format!("... [{book_id}].fb2")));

    Ok(())
}

#[test]
fn test_extract_books_keeps_short_author_with_long_title() -> Result<()> {
    const MAX_FILE_NAME_LEN: usize = 255;

    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let title = "Очень длинное, длинное, длинное название для книги, содержащее много-много букв и разных символов. ".repeat(4);
    let book_id = 456_123;
    let conn = init_db(&db_path);
    populate_db(&conn, "archive.zip", &[(book_id, &title, "Джек", "Джон")]);
    drop(conn);

    create_test_zip(
        lib_path,
        "archive.zip",
        &[(book_id, &title, "Джек", "Джон")],
    );

    let output_dir = tempfile::tempdir()?;
    extract_books(lib_path, &[book_id], output_dir.path())?;

    let entries: Vec<_> = fs::read_dir(output_dir.path())?
        .filter_map(std::result::Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(
        filename.len() <= MAX_FILE_NAME_LEN,
        "filename is {} bytes: {filename}",
        filename.len()
    );
    assert!(
        filename.starts_with("Джек Джон - "),
        "filename should keep the short author: {filename}"
    );
    assert!(filename.ends_with(&format!("... [{book_id}].fb2")));

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
