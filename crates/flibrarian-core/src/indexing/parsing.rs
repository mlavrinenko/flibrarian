use anyhow::{Context, Result};
use log::trace;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use zip::ZipArchive;

use super::check_cancelled;
use super::extract_fields::{
    extract_authors, extract_date, extract_genres, extract_lang, extract_sequence, extract_title,
};
use super::types::{Book, IndexingPhase, PartialFictionBook};
use super::xml::{
    build_template, create_template_with_description, extract_description_bytes,
    extract_xml_declaration_and_fictionbook_tag,
};
use crate::encoding::bytes_to_utf8_string;

pub fn parse_zip_archive(
    zip_path: &Path,
    books_completed: &AtomicUsize,
    books_total: usize,
    on_progress: &(dyn Fn(IndexingPhase, usize, usize) + Send + Sync),
    on_warning: &(dyn Fn(&str) + Send + Sync),
    on_info: &(dyn Fn(&str) + Send + Sync),
    cancelled: &AtomicBool,
) -> Option<(String, Vec<Book>)> {
    check_cancelled(cancelled).ok()?;
    let zip_filename = zip_path.file_name().and_then(|n| n.to_str())?;

    log::info!("Parsing zip {}", zip_path.display());
    on_info(&format!("Processing archive {zip_filename}"));

    let books = match read_books_from_zip(
        zip_path,
        zip_filename,
        books_completed,
        books_total,
        on_progress,
        on_warning,
        cancelled,
    ) {
        Ok(books) => books,
        Err(e) => {
            let msg = format!("Failed to parse zip {}: {e}", zip_path.display());
            log::warn!("{msg}");
            on_warning(&msg);
            return None;
        }
    };

    if books.is_empty() {
        log::info!("No FB2 files or valid data found in {zip_filename}. Skipping.");
        return None;
    }

    on_info(&format!("Loaded {} books from {zip_filename}", books.len()));
    Some((zip_filename.to_string(), books))
}

fn read_books_from_zip(
    zip_path: &Path,
    zip_filename: &str,
    books_completed: &AtomicUsize,
    books_total: usize,
    on_progress: &(dyn Fn(IndexingPhase, usize, usize) + Send + Sync),
    on_warning: &(dyn Fn(&str) + Send + Sync),
    cancelled: &AtomicBool,
) -> Result<Vec<Book>> {
    let file = fs::File::open(zip_path).context("Failed to open zip file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read zip archive")?;

    let fb2_entries: Vec<(usize, String)> = (0..archive.len())
        .filter_map(|i| {
            let name = archive.name_for_index(i)?;
            if Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("fb2"))
            {
                Some((i, name.to_string()))
            } else {
                None
            }
        })
        .collect();

    Ok(fb2_entries
        .iter()
        .filter_map(|(index, filepath)| {
            check_cancelled(cancelled).ok()?;
            trace!("process_fb2_file: {zip_filename}/{filepath}");
            let result = match read_book_from_archive(&mut archive, *index) {
                Ok(data) => Some(data),
                Err(e) => {
                    let msg = format!(
                        "Failed to process FB2 file {filepath} (index {index}) in {zip_filename}: {e}"
                    );
                    log::warn!("{msg}");
                    on_warning(&msg);
                    None
                }
            };
            let current = books_completed.fetch_add(1, Ordering::Relaxed) + 1;
            on_progress(IndexingPhase::Parsing, current, books_total);
            result
        })
        .collect())
}

fn read_book_from_archive(archive: &mut ZipArchive<fs::File>, file_index: usize) -> Result<Book> {
    let mut fb2_file = archive.by_index(file_index)?;
    if fb2_file.size() == 0 {
        return Err(anyhow::anyhow!("FB2 file is empty, skipping it"));
    }

    let id = Book::parse_id(fb2_file.name())?;
    let mut raw_bytes = Vec::with_capacity(usize::try_from(fb2_file.size()).unwrap_or(0));
    fb2_file.read_to_end(&mut raw_bytes)?;

    parse_book_from_bytes(id, &raw_bytes)
}

pub fn parse_book_from_bytes(id: u32, raw_bytes: &[u8]) -> Result<Book> {
    let fiction_book = deserialize_fb2(raw_bytes)?;

    let first_title_info = fiction_book
        .description
        .title_info
        .first()
        .context("No title-info found in FB2 file")?;

    Ok(Book {
        id,
        title: extract_title(first_title_info),
        genres: extract_genres(first_title_info),
        authors: extract_authors(first_title_info),
        date: extract_date(first_title_info, &fiction_book.description),
        lang: extract_lang(first_title_info),
        file_size: raw_bytes.len() as u64,
        sequence: extract_sequence(first_title_info),
    })
}

pub fn deserialize_fb2(raw_bytes: &[u8]) -> Result<PartialFictionBook> {
    let (xml_decl, fictionbook_open) = extract_xml_declaration_and_fictionbook_tag(raw_bytes);

    if let Some(desc_bytes) = extract_description_bytes(raw_bytes) {
        let description = if let Ok(s) = std::str::from_utf8(desc_bytes) {
            s.to_string()
        } else {
            convert_bytes_to_utf8(desc_bytes)?
        };

        let template = build_template(&xml_decl, &fictionbook_open, &description);
        return Ok(quick_xml::de::from_str(&template)?);
    }

    deserialize_fb2_full(raw_bytes, &xml_decl, &fictionbook_open)
}

fn convert_bytes_to_utf8(raw_bytes: &[u8]) -> Result<String> {
    bytes_to_utf8_string(raw_bytes).context("Failed to convert bytes to UTF-8")
}

fn deserialize_fb2_full(
    raw_bytes: &[u8],
    xml_decl: &str,
    fictionbook_open: &str,
) -> Result<PartialFictionBook> {
    let utf8_content = convert_bytes_to_utf8(raw_bytes)?;
    let processed_content =
        create_template_with_description(&utf8_content, xml_decl, fictionbook_open);
    Ok(quick_xml::de::from_str(&processed_content)?)
}
