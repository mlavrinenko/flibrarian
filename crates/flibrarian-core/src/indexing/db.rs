use anyhow::{Context, Result};
use duckdb::Error as DuckDbError;
use duckdb::{AccessMode, Connection, params};
use log::info;
use serde_json;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use super::check_cancelled;
use super::types::{ArchiveStatus, Book, IndexingPhase};
use crate::common::db_config;

pub fn write_books_to_db(db_path: &Path, zip_filename: &str, books: &[Book]) -> Result<()> {
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite)?)?;
    write_books_to_db_conn(&conn, zip_filename, books)
}

pub fn write_books_to_db_conn(conn: &Connection, zip_filename: &str, books: &[Book]) -> Result<()> {
    mark_archive_as_indexing(conn, zip_filename)?;
    delete_existing_books_for_archive(conn, zip_filename)?;

    let archive_id = upsert_archive_direct(conn, zip_filename)?;
    insert_books_bulk(conn, books, archive_id)?;

    conn.execute(
        "UPDATE archives SET status = ? WHERE id = ?",
        params![ArchiveStatus::Indexed, archive_id],
    )?;

    log::info!(
        "Successfully indexed {} books from {}",
        books.len(),
        zip_filename
    );

    Ok(())
}

fn mark_archive_as_indexing(conn: &Connection, zip_filename: &str) -> Result<()> {
    let updated = conn.execute(
        "UPDATE archives SET status = ? WHERE name = ?",
        params![ArchiveStatus::Indexing, zip_filename],
    )?;

    if updated > 0 {
        info!("Marked archive {zip_filename} as indexing before delete");
    }

    Ok(())
}

fn delete_existing_books_for_archive(conn: &Connection, zip_filename: &str) -> Result<()> {
    let archive_id: Option<u32> = conn
        .query_row(
            "SELECT id FROM archives WHERE name = ?",
            params![zip_filename],
            |row| row.get(0),
        )
        .ok();

    if let Some(arch_id) = archive_id {
        let deleted_ba = conn
            .execute(
                "DELETE FROM books_authors WHERE book_id IN (SELECT id FROM books WHERE archive_id = ?)",
                params![arch_id],
            )
            .context("Failed to delete books_authors for archive")?;
        info!("{deleted_ba} books_authors deleted");

        let deleted_books = conn
            .execute("DELETE FROM books WHERE archive_id = ?", params![arch_id])
            .context("Failed to delete books for archive")?;
        info!("{deleted_books} books deleted");

        let deleted_authors = conn
            .execute(
                "DELETE FROM authors WHERE NOT EXISTS (SELECT 1 FROM books_authors WHERE books_authors.author_id = authors.id)",
                [],
            )
            .context("Failed to delete orphaned authors")?;
        if deleted_authors > 0 {
            info!("{deleted_authors} orphaned authors deleted");
        }
    }

    Ok(())
}

fn upsert_archive_direct(conn: &Connection, zip_filename: &str) -> Result<u32> {
    if let Ok(id) = conn.query_row(
        "SELECT id FROM archives WHERE name = ?",
        params![zip_filename],
        |row| row.get(0),
    ) {
        conn.execute(
            "UPDATE archives SET status = ? WHERE id = ?",
            params![ArchiveStatus::Indexing, id],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO archives (name, status) VALUES (?, ?)",
            params![zip_filename, ArchiveStatus::Indexing],
        )?;
        conn.query_row(
            "SELECT id FROM archives WHERE name = ?",
            params![zip_filename],
            |row| row.get(0),
        )
        .context("Failed to retrieve archive id after insert")
    }
}

fn insert_books_bulk(conn: &Connection, books: &[Book], archive_id: u32) -> Result<()> {
    conn.execute_batch(
        "CREATE TEMPORARY TABLE IF NOT EXISTS staging_books (
            id UINTEGER, title VARCHAR, genres JSON, date VARCHAR,
            lang VARCHAR, file_size UBIGINT, sequence VARCHAR, archive_id UINTEGER
        )",
    )?;
    conn.execute_batch(
        "CREATE TEMPORARY TABLE IF NOT EXISTS staging_authors (
            id VARCHAR, first_name VARCHAR, middle_name VARCHAR,
            last_name VARCHAR, nickname VARCHAR
        )",
    )?;
    conn.execute_batch(
        "CREATE TEMPORARY TABLE IF NOT EXISTS staging_ba (
            book_id UINTEGER, author_id VARCHAR
        )",
    )?;

    conn.execute_batch(
        "DELETE FROM staging_books; DELETE FROM staging_authors; DELETE FROM staging_ba",
    )?;

    {
        let mut app_books = conn.appender("staging_books")?;
        let mut app_authors = conn.appender("staging_authors")?;
        let mut app_ba = conn.appender("staging_ba")?;

        for book in books {
            let genres_json =
                serde_json::to_string(&book.genres).context("Failed to serialize genres")?;
            app_books.append_row(params![
                book.id,
                book.title,
                genres_json,
                book.date,
                book.lang,
                book.file_size,
                book.sequence,
                archive_id
            ])?;

            for author in &book.authors {
                app_authors.append_row(params![
                    &author.id,
                    &author.first_name,
                    &author.middle_name,
                    &author.last_name,
                    &author.nickname
                ])?;
                app_ba.append_row(params![book.id, &author.id])?;
            }
        }
    }

    conn.execute_batch(
        "INSERT OR IGNORE INTO authors SELECT DISTINCT * FROM staging_authors;
         INSERT OR REPLACE INTO books SELECT * FROM staging_books;
         INSERT OR REPLACE INTO books_authors SELECT * FROM staging_ba",
    )?;

    Ok(())
}

pub fn get_ignored_archives(db_path: &Path) -> Result<HashSet<String>> {
    log::info!("Reading indexed archives from database");
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadOnly)?)?;

    let indexed_names: Result<HashSet<String>, DuckDbError> = conn
        .prepare("SELECT name FROM archives WHERE status = ?")?
        .query_map(params![ArchiveStatus::Indexed], |row| row.get(0))?
        .collect();

    log::info!(
        "Found {} indexed archives.",
        indexed_names
            .as_ref()
            .map_or(0, std::collections::HashSet::len)
    );

    indexed_names.map_err(std::convert::Into::into)
}

pub fn get_all_archives(db_path: &Path) -> Result<Vec<String>> {
    log::info!("Reading all archives from database");
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadOnly)?)?;

    let all_names: Result<Vec<String>, DuckDbError> = conn
        .prepare("SELECT name FROM archives")?
        .query_map([], |row| row.get(0))?
        .collect();

    all_names.map_err(std::convert::Into::into)
}

pub fn create_search_index(
    conn: &Connection,
    on_progress: &(dyn Fn(IndexingPhase, usize, usize) + Send + Sync),
    cancelled: &AtomicBool,
) -> Result<()> {
    check_cancelled(cancelled)?;

    on_progress(IndexingPhase::BuildingSearchIndex, 0, 1);

    conn.execute_batch("DROP TABLE IF EXISTS search_index_new")
        .context("Failed to drop search_index_new table")?;

    conn.execute_batch(
        r"
        CREATE TABLE search_index_new AS
        SELECT
            book.id,
            book.title,
            (
                SELECT string_agg(json_extract(g.value, '$')::VARCHAR, ', ')
                FROM json_each(book.genres::JSON) AS g
            ) AS genres_text,
            (
                SELECT string_agg(
                    COALESCE(author.first_name || ' ', '') ||
                    COALESCE(author.middle_name || ' ', '') ||
                    COALESCE(author.last_name || ' ', '') ||
                    COALESCE(' (' || author.nickname || ')', '')
                )
                FROM authors author
                JOIN books_authors ba ON author.id = ba.author_id
                WHERE ba.book_id = book.id
            ) AS authors_text,
            book.sequence
        FROM books book
        ",
    )
    .context("Failed to create search_index_new with data")?;

    on_progress(IndexingPhase::BuildingSearchIndex, 1, 1);

    conn.execute_batch("DROP TABLE IF EXISTS search_index")
        .context("Failed to drop old search_index table")?;

    conn.execute_batch("ALTER TABLE search_index_new RENAME TO search_index")
        .context("Failed to rename search_index_new to search_index")?;

    check_cancelled(cancelled)?;
    on_progress(IndexingPhase::CreatingFtsIndex, 0, 0);

    conn.execute_batch(
        "
        PRAGMA create_fts_index(
            'search_index', 'id',
            'title', 'authors_text', 'sequence',
            overwrite = 1,
            stemmer = 'russian'
        )
    ",
    )
    .context("Failed to create FTS index on search_index table")?;

    Ok(())
}

pub fn is_search_index_valid(conn: &Connection, total_books: usize) -> bool {
    if total_books == 0 {
        return true;
    }

    let has_table: bool = conn
        .prepare("SELECT table_name FROM information_schema.tables WHERE table_name='search_index'")
        .and_then(|mut stmt| stmt.query_row([], |row| row.get::<usize, String>(0)))
        .is_ok();

    if !has_table {
        return false;
    }

    let index_count: usize = conn
        .query_row("SELECT COUNT(*) FROM search_index", [], |row| row.get(0))
        .unwrap_or(0);

    index_count == total_books
}
