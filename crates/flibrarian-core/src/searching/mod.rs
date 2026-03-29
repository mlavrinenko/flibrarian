mod filters;
mod queries;

use anyhow::{Context, Result};
use duckdb::{AccessMode, Connection, OptionalExt};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::atomic::AtomicBool;

use crate::common::{db_config, get_db_path};
use crate::indexing::{Author, IndexingPhase, create_search_index};

pub use queries::{value_ref_to_vec_author_struct, value_ref_to_vec_string};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct SearchFilters {
    pub title: Option<String>,
    pub authors: Option<String>,
    pub genres: Option<String>,
    pub date: Option<String>,
    pub lang: Option<String>,
    pub file_size: Option<String>,
    pub sequence: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LanguageCount {
    pub lang: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct FoundBook {
    pub id: u32,
    pub title: String,
    pub genres: Vec<String>,
    pub authors: Vec<Author>,
    pub date: String,
    pub lang: String,
    pub file_size: u64,
    pub sequence: String,
    pub score: f64,
}

pub fn get_languages(library_path: &Path) -> Result<Vec<LanguageCount>> {
    let db_path = get_db_path(library_path);
    if !db_path.exists() {
        return Ok(Vec::new());
    }
    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)
        .context("Failed to open database for languages")?;

    let mut stmt = conn
        .prepare(
            "SELECT lang, COUNT(*) as cnt FROM books \
             WHERE lang IS NOT NULL AND lang != '' \
             GROUP BY lang ORDER BY cnt DESC",
        )
        .context("Failed to prepare languages query")?;

    let languages = stmt
        .query_map([], |row| {
            Ok(LanguageCount {
                lang: row.get(0)?,
                count: row.get(1)?,
            })
        })
        .context("Failed to execute languages query")?
        .filter_map(Result::ok)
        .collect();

    Ok(languages)
}

pub fn get_book_count(library_path: &Path) -> Result<u64> {
    let db_path = get_db_path(library_path);
    if !db_path.exists() {
        return Ok(0);
    }
    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)
        .context("Failed to open database for book count")?;
    conn.query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
        .context("Failed to count books")
}

pub fn search_library(
    library_path: &Path,
    query: &str,
    filters: &SearchFilters,
) -> Result<Vec<FoundBook>> {
    let db_path = get_db_path(library_path);
    let open_err = |mode: &str| {
        format!(
            "Failed to open database ({mode}) for search from {}",
            db_path.to_str().unwrap_or(&format!(
                "unresolved path from {}",
                library_path.to_str().unwrap_or("unresolved input")
            ))
        )
    };

    if query.trim().is_empty() {
        let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)
            .context(open_err("read-only"))?;
        queries::execute_filter_only_query(&conn, filters)
    } else {
        let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)
            .context(open_err("read-only"))?;
        if has_search_index(&conn)? {
            conn.execute_batch("LOAD fts;")
                .context("Failed to load FTS extension")?;
            queries::execute_search_query(&conn, query, filters)
        } else {
            drop(conn);
            let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite)?)
                .context(open_err("read-write"))?;
            create_search_index(&conn, &|_: IndexingPhase, _, _| {}, &AtomicBool::new(false))?;
            queries::execute_search_query(&conn, query, filters)
        }
    }
}

fn has_search_index(conn: &Connection) -> Result<bool> {
    conn.execute_batch("LOAD fts;")
        .context("Failed to load FTS extension")?;

    let exists = conn
        .prepare("SELECT table_name FROM information_schema.tables WHERE table_name='search_index'")
        .context("Failed to check for search_index table")?
        .query_row([], |row| row.get::<usize, String>(0))
        .optional()
        .context("Failed to execute query for search_index table")?
        .is_some();
    Ok(exists)
}
