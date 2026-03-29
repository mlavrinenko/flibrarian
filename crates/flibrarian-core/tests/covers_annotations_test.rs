use anyhow::Result;
use duckdb::{AccessMode, Connection, params};
use flibrarian_core::annotations::get_book_annotation;
use flibrarian_core::common::db_config;
use flibrarian_core::covers::get_book_cover;
use std::io::Write;
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn init_db(db_path: &Path) -> Connection {
    let conn =
        Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite).unwrap()).unwrap();
    conn.execute_batch(include_str!("../src/schema.sql"))
        .unwrap();
    conn
}

fn insert_book_record(conn: &Connection, archive_name: &str, book_id: u32) {
    conn.execute(
        "INSERT OR IGNORE INTO archives (name, status) VALUES (?, 'indexed')",
        params![archive_name],
    )
    .unwrap();

    let archive_id: u32 = conn
        .query_row(
            "SELECT id FROM archives WHERE name = ?",
            params![archive_name],
            |row| row.get(0),
        )
        .unwrap();

    conn.execute(
        "INSERT OR REPLACE INTO books (id, title, archive_id) VALUES (?, 'Test Book', ?)",
        params![book_id, archive_id],
    )
    .unwrap();
}

fn create_zip_with_fb2(dir: &Path, zip_name: &str, book_id: u32, fb2_content: &[u8]) {
    let zip_path = dir.join(zip_name);
    let file = std::fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default();

    zip.start_file(format!("{book_id}.fb2"), options).unwrap();
    zip.write_all(fb2_content).unwrap();
    zip.finish().unwrap();
}

#[test]
fn get_book_annotation_with_annotation() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "archive.zip", 1);
    drop(conn);

    let fb2 = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>Annotated Book</book-title>
<annotation><p>This is the book annotation.</p><p>Second paragraph.</p></annotation>
</title-info>
</description>
<body><section><p>Content</p></section></body>
</FictionBook>"#;

    create_zip_with_fb2(lib_path, "archive.zip", 1, fb2);

    let annotation = get_book_annotation(lib_path, 1)?;
    assert!(annotation.is_some());
    assert!(annotation.unwrap().contains("book annotation"));

    Ok(())
}

#[test]
fn get_book_annotation_no_annotation() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "archive.zip", 1);
    drop(conn);

    let fb2 = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>No Annotation</book-title>
</title-info>
</description>
<body><section><p>Content</p></section></body>
</FictionBook>"#;

    create_zip_with_fb2(lib_path, "archive.zip", 1, fb2);

    let annotation = get_book_annotation(lib_path, 1)?;
    assert!(annotation.is_none());

    Ok(())
}

#[test]
fn get_book_annotation_book_not_in_db() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);

    let annotation = get_book_annotation(lib_path, 999)?;
    assert!(annotation.is_none());

    Ok(())
}

#[test]
fn get_book_annotation_missing_archive_file() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "missing.zip", 1);
    drop(conn);

    let annotation = get_book_annotation(lib_path, 1)?;
    assert!(annotation.is_none());

    Ok(())
}

#[test]
fn get_book_cover_with_cover() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "archive.zip", 1);
    drop(conn);

    let fb2 = br##"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0" xmlns:l="http://www.w3.org/1999/xlink">
<description>
<title-info>
<book-title>Book With Cover</book-title>
<coverpage><image l:href="#cover.jpg"/></coverpage>
</title-info>
</description>
<body><section><p>Content</p></section></body>
<binary id="cover.jpg" content-type="image/jpeg">SGVsbG8gV29ybGQ=</binary>
</FictionBook>"##;

    create_zip_with_fb2(lib_path, "archive.zip", 1, fb2);

    let cover = get_book_cover(lib_path, 1)?;
    assert!(cover.is_some());
    let cover = cover.unwrap();
    assert_eq!(cover.content_type, "image/jpeg");
    assert_eq!(cover.data, b"Hello World");

    Ok(())
}

#[test]
fn get_book_cover_no_coverpage() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "archive.zip", 1);
    drop(conn);

    let fb2 = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>No Cover</book-title>
</title-info>
</description>
<body><section><p>Content</p></section></body>
</FictionBook>"#;

    create_zip_with_fb2(lib_path, "archive.zip", 1, fb2);

    let cover = get_book_cover(lib_path, 1)?;
    assert!(cover.is_none());

    Ok(())
}

#[test]
fn get_book_cover_book_not_in_db() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");
    init_db(&db_path);

    let cover = get_book_cover(lib_path, 999)?;
    assert!(cover.is_none());

    Ok(())
}

#[test]
fn get_book_cover_missing_archive() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "missing.zip", 1);
    drop(conn);

    let cover = get_book_cover(lib_path, 1)?;
    assert!(cover.is_none());

    Ok(())
}

#[test]
fn get_book_cover_book_not_in_archive() -> Result<()> {
    let tmp = tempfile::tempdir()?;
    let lib_path = tmp.path();
    let db_path = lib_path.join("lib.duckdb");

    let conn = init_db(&db_path);
    insert_book_record(&conn, "archive.zip", 99);
    drop(conn);

    let fb2 = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description><title-info><book-title>X</book-title></title-info></description>
<body></body>
</FictionBook>"#;

    create_zip_with_fb2(lib_path, "archive.zip", 1, fb2);

    let cover = get_book_cover(lib_path, 99)?;
    assert!(cover.is_none());

    Ok(())
}
