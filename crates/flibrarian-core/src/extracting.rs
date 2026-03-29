use crate::common::{db_config, get_db_path};
use anyhow::{Context, Result};
use duckdb::{AccessMode, Connection, params_from_iter};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs::File, io::copy};
use zip::ZipArchive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedBook {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone)]
struct BookOutput {
    name: String,
    author: Option<String>,
    archive: String,
}

type BookMetadata = (HashMap<u32, BookOutput>, HashMap<String, HashSet<u32>>);

pub fn extract_books(
    library_path: &Path,
    book_ids: &[u32],
    output_dir: &Path,
) -> Result<Vec<ExtractedBook>> {
    let (book_map, books_by_archive) = query_book_metadata(library_path, book_ids)?;

    info!("Extracting books to directory: {}", output_dir.display());

    let mut extracted = Vec::new();
    for (archive_name, archive_book_ids) in books_by_archive {
        let mut results = extract_from_archive(
            library_path,
            &archive_name,
            &archive_book_ids,
            &book_map,
            output_dir,
        )?;
        extracted.append(&mut results);
    }

    Ok(extracted)
}

fn extract_from_archive(
    library_path: &Path,
    archive_name: &str,
    archive_book_ids: &HashSet<u32>,
    book_map: &HashMap<u32, BookOutput>,
    output_dir: &Path,
) -> Result<Vec<ExtractedBook>> {
    let zip_path = library_path.join(archive_name);
    info!("Processing archive: {}", zip_path.display());

    let file = match File::open(&zip_path) {
        Ok(f) => f,
        Err(err) => {
            error!("Failed to open zip archive {}: {err}", zip_path.display());
            return Ok(Vec::new());
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(ar) => ar,
        Err(err) => {
            error!("Failed to read zip archive {}: {err}", zip_path.display());
            return Ok(Vec::new());
        }
    };

    let mut extracted = Vec::new();

    for &archive_book_id in archive_book_ids {
        if let Some(book) = extract_single_book(
            &mut archive,
            archive_name,
            archive_book_id,
            book_map,
            output_dir,
        )? {
            extracted.push(book);
        }
    }

    Ok(extracted)
}

fn extract_single_book(
    archive: &mut ZipArchive<File>,
    archive_name: &str,
    book_id: u32,
    book_map: &HashMap<u32, BookOutput>,
    output_dir: &Path,
) -> Result<Option<ExtractedBook>> {
    let book_name_file = format!("{book_id}.fb2");

    let (file_name, title, author) = book_map.get(&book_id).map_or_else(
        || (format!("{book_id}.fb2"), String::new(), String::new()),
        |book_data| {
            let author_str = book_data
                .author
                .as_deref()
                .unwrap_or_default()
                .trim()
                .replace(" ,", ", ");

            let file_name = format!("{author_str} - {} [{book_id}].fb2", book_data.name);
            (file_name, book_data.name.clone(), author_str)
        },
    );

    let mut file = match archive.by_name(&book_name_file) {
        Ok(file) => file,
        Err(err) => {
            warn!("Archive {archive_name} does not contain book {book_name_file}: {err}");
            return Ok(None);
        }
    };

    let saved_book_path = output_dir.join(&file_name);

    let mut saved_book_file = File::create(&saved_book_path).context(format!(
        "Failed to create output file {}",
        saved_book_path.display()
    ))?;

    copy(&mut file, &mut saved_book_file).context(format!(
        "Failed to write content to file {}",
        saved_book_path.display()
    ))?;

    info!("Successfully extracted {}", saved_book_path.display());

    Ok(Some(ExtractedBook {
        id: book_id,
        title,
        author,
        output_path: saved_book_path,
    }))
}

#[derive(Debug, Clone)]
pub struct ExtractedBookContent {
    pub id: u32,
    pub file_name: String,
    pub data: Vec<u8>,
}

pub fn extract_book_contents(
    library_path: &Path,
    book_ids: &[u32],
) -> Result<Vec<ExtractedBookContent>> {
    let (book_map, books_by_archive) = query_book_metadata(library_path, book_ids)?;

    let mut results = Vec::new();
    for (archive_name, ids) in books_by_archive {
        let Some(mut archive) = open_zip_archive(library_path, &archive_name) else {
            continue;
        };
        read_books_from_archive(&mut archive, &archive_name, &ids, &book_map, &mut results)?;
    }

    Ok(results)
}

fn read_books_from_archive(
    archive: &mut ZipArchive<File>,
    archive_name: &str,
    ids: &HashSet<u32>,
    book_map: &HashMap<u32, BookOutput>,
    results: &mut Vec<ExtractedBookContent>,
) -> Result<()> {
    for &book_id in ids {
        let book_name_file = format!("{book_id}.fb2");
        let file_name = make_book_file_name(book_id, book_map);

        let mut zip_file = match archive.by_name(&book_name_file) {
            Ok(f) => f,
            Err(err) => {
                warn!("Archive {archive_name} missing {book_name_file}: {err}");
                continue;
            }
        };

        let mut data = Vec::new();
        zip_file.read_to_end(&mut data).context(format!(
            "Failed to read {book_name_file} from {archive_name}"
        ))?;

        results.push(ExtractedBookContent {
            id: book_id,
            file_name,
            data,
        });
    }
    Ok(())
}

fn make_book_file_name(book_id: u32, book_map: &HashMap<u32, BookOutput>) -> String {
    book_map.get(&book_id).map_or_else(
        || format!("{book_id}.fb2"),
        |b| {
            let author_str = b
                .author
                .as_deref()
                .unwrap_or_default()
                .trim()
                .replace(" ,", ", ");
            format!("{author_str} - {} [{book_id}].fb2", b.name)
        },
    )
}

fn open_zip_archive(library_path: &Path, archive_name: &str) -> Option<ZipArchive<File>> {
    let zip_path = library_path.join(archive_name);
    let file = match File::open(&zip_path) {
        Ok(f) => f,
        Err(err) => {
            error!("Failed to open zip archive {}: {err}", zip_path.display());
            return None;
        }
    };
    match ZipArchive::new(file) {
        Ok(ar) => Some(ar),
        Err(err) => {
            error!("Failed to read zip archive {}: {err}", zip_path.display());
            None
        }
    }
}

fn query_book_metadata(library_path: &Path, book_ids: &[u32]) -> Result<BookMetadata> {
    let db_path = get_db_path(library_path);
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadOnly)?)
        .context("Failed to open database")?;

    let placeholders = book_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<&str>>()
        .join(",");

    let select_query = format!(
        r"
        SELECT
            book.id,
            book.title,
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
            a.name AS archive_name
        FROM books book
        JOIN archives a ON book.archive_id = a.id
        WHERE book.id IN ({placeholders});
    "
    );

    let mut stmt = conn
        .prepare(&select_query)
        .context("Failed to prepare statement for selecting books")?;

    let book_map: HashMap<u32, BookOutput> = stmt
        .query_map(params_from_iter(book_ids.iter().copied()), |row| {
            let id = row.get(0)?;
            let title: String = row.get(1)?;
            let authors_text: Option<String> = row.get(2)?;
            let archive: String = row.get(3)?;
            Ok((
                id,
                BookOutput {
                    name: title,
                    author: authors_text,
                    archive,
                },
            ))
        })?
        .map(|res| res.context("Failed to get book details from row"))
        .collect::<Result<_>>()?;

    let mut books_by_archive: HashMap<String, HashSet<u32>> = HashMap::new();
    for (&book_id, book_output) in &book_map {
        if !book_output.archive.is_empty() {
            books_by_archive
                .entry(book_output.archive.clone())
                .or_default()
                .insert(book_id);
        }
    }

    Ok((book_map, books_by_archive))
}
