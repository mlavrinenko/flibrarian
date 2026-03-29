use anyhow::{Context, Result};
use duckdb::Connection;
use duckdb::types::{Value, ValueRef};
use log::warn;

use super::filters::build_filter_conditions;
use super::{FoundBook, SearchFilters};
use crate::indexing::Author;

pub fn execute_filter_only_query(
    conn: &Connection,
    filters: &SearchFilters,
) -> Result<Vec<FoundBook>> {
    let (filter_sql, filter_params) = build_filter_conditions(filters, "b");

    let sql = format!(
        "
                SELECT
                    b.id,
                    b.title,
                    b.genres,
                    (
                        SELECT json_group_array(
                            json_object(
                                'id', author.id,
                                'first_name', author.first_name,
                                'middle_name', author.middle_name,
                                'last_name', author.last_name,
                                'nickname', author.nickname
                            )
                        )
                        FROM authors author
                        JOIN books_authors ba ON author.id = ba.author_id
                        WHERE ba.book_id = b.id
                    ) AS authors,
                    b.date,
                    b.lang,
                    b.file_size,
                    b.sequence,
                    0.0 AS score
                FROM books b
                WHERE 1=1
                {filter_sql}
                ORDER BY b.title
                LIMIT 1000;
            "
    );

    let mut param_values: Vec<Box<dyn duckdb::ToSql>> = Vec::new();
    for p in filter_params {
        param_values.push(Box::new(p));
    }
    let param_refs: Vec<&dyn duckdb::ToSql> = param_values.iter().map(AsRef::as_ref).collect();

    let results = conn
        .prepare(&sql)
        .context("Failed to prepare filter query")?
        .query_map(param_refs.as_slice(), map_row_to_found_book)
        .context("Failed to execute filter query")?
        .filter_map(|r| match r {
            Ok(book) => Some(book),
            Err(err) => {
                warn!("Failed to deserialize filter result row: {err}");
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(results)
}

pub fn execute_search_query(
    conn: &Connection,
    query: &str,
    filters: &SearchFilters,
) -> Result<Vec<FoundBook>> {
    let (filter_sql, filter_params) = build_filter_conditions(filters, "found_book");

    let sql = format!(
        "
                SELECT
                    found_book.id,
                    found_book.title,
                    (
                        SELECT book.genres
                        FROM books book
                        WHERE found_book.id = book.id
                    ) AS genres,
                    (
                        SELECT json_group_array(
                            json_object(
                                'id', author.id,
                                'first_name', author.first_name,
                                'middle_name', author.middle_name,
                                'last_name', author.last_name,
                                'nickname', author.nickname
                            )
                        )
                        FROM authors author
                        JOIN books_authors ba ON author.id = ba.author_id
                        WHERE ba.book_id = found_book.id
                    ) AS authors,
                    (
                        SELECT book.date
                        FROM books book
                        WHERE found_book.id = book.id
                    ) AS date,
                    (
                        SELECT book.lang
                        FROM books book
                        WHERE found_book.id = book.id
                    ) AS lang,
                    (
                        SELECT book.file_size
                        FROM books book
                        WHERE found_book.id = book.id
                    ) AS file_size,
                    found_book.sequence,
                    found_book.score
                FROM (
                    SELECT id, title, sequence, fts_main_search_index.match_bm25(id, ?) AS score
                    FROM search_index
                ) found_book
                WHERE found_book.score IS NOT NULL
                {filter_sql}
                ORDER BY found_book.score DESC
                LIMIT 1000;
            "
    );

    let mut param_values: Vec<Box<dyn duckdb::ToSql>> = Vec::new();
    param_values.push(Box::new(query.to_string()));
    for p in filter_params {
        param_values.push(Box::new(p));
    }
    let param_refs: Vec<&dyn duckdb::ToSql> = param_values.iter().map(AsRef::as_ref).collect();

    let results = conn
        .prepare(&sql)
        .context("Failed to prepare search query")?
        .query_map(param_refs.as_slice(), map_row_to_found_book)
        .context("Failed to execute search query")?
        .filter_map(|r| match r {
            Ok(book) => Some(book),
            Err(err) => {
                warn!("Failed to deserialize search result row: {err}");
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(results)
}

fn map_row_to_found_book(row: &duckdb::Row) -> duckdb::Result<FoundBook> {
    let id: u32 = row.get("id")?;
    let title: String = row.get("title")?;
    let genres =
        value_ref_to_vec_string(row.get_ref("genres")?).map_err(|e| conversion_error(2, &e))?;
    let authors = value_ref_to_vec_author_struct(row.get_ref("authors")?)
        .map_err(|e| conversion_error(3, &e))?;
    let date: String = row.get("date")?;
    let lang: String = row.get("lang")?;
    let file_size: u64 = row.get("file_size")?;
    let sequence: String = row.get("sequence")?;
    let score: f64 = row.get("score")?;

    Ok(FoundBook {
        id,
        title,
        genres,
        authors,
        date,
        lang,
        file_size,
        sequence,
        score,
    })
}

fn conversion_error(col: usize, err: &anyhow::Error) -> duckdb::Error {
    duckdb::Error::FromSqlConversionFailure(
        col,
        duckdb::types::Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            err.to_string(),
        )),
    )
}

pub fn value_ref_to_vec_string(value: ValueRef) -> Result<Vec<String>> {
    let Value::Text(json_str) = value.to_owned() else {
        return Err(anyhow::anyhow!("Expected a Text value (JSON string)"));
    };
    serde_json::from_str(&json_str).context("Failed to deserialize JSON string to Vec<String>")
}

pub fn value_ref_to_vec_author_struct(value: ValueRef) -> Result<Vec<Author>> {
    let json_str = match value.to_owned() {
        Value::Null => return Ok(Vec::new()),
        Value::Text(text) => text,
        _ => return Err(anyhow::anyhow!("Expected a Text value (JSON string)")),
    };
    serde_json::from_str(&json_str)
        .context("Failed to deserialize JSON string to Vec<AuthorStruct>")
}
