#![allow(dead_code)]

use duckdb::{AccessMode, Connection, params};
use flibrarian_core::common::db_config;
use std::fs;
use std::io::Write;
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

pub fn minimal_fb2(id: u32, title: &str, author_first: &str, author_last: &str) -> Vec<u8> {
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

pub fn create_test_zip(dir: &Path, zip_name: &str, books: &[(u32, &str, &str, &str)]) {
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

pub fn init_db(db_path: &Path) -> Connection {
    let conn =
        Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite).unwrap()).unwrap();
    conn.execute_batch(include_str!("../../src/schema.sql"))
        .unwrap();
    conn
}

pub fn populate_db(conn: &Connection, archive_name: &str, books: &[(u32, &str, &str, &str)]) {
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

pub fn add_author(conn: &Connection, book_id: u32, author_id: &str, first_name: &str) {
    conn.execute(
        "INSERT OR IGNORE INTO authors (id, first_name, middle_name, last_name, nickname) VALUES (?, ?, NULL, NULL, NULL)",
        params![author_id, first_name],
    )
    .unwrap();
    conn.execute(
        "INSERT OR REPLACE INTO books_authors (book_id, author_id) VALUES (?, ?)",
        params![book_id, author_id],
    )
    .unwrap();
}

pub fn populate_db_without_authors(conn: &Connection, archive_name: &str, books: &[(u32, &str)]) {
    conn.execute(
        "INSERT INTO archives (name, status) VALUES (?, 'indexed')",
        params![archive_name],
    )
    .unwrap();
    for (id, title) in books {
        conn.execute(
            "INSERT INTO books (id, title, genres, date, sequence, archive_id) VALUES (?, ?, '[]', '2023', NULL, (SELECT id FROM archives WHERE name = ?))",
            params![id, title, archive_name],
        )
        .unwrap();
    }
}
