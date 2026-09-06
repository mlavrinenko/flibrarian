mod db;
mod events;
mod extract_fields;
mod parsing;
pub mod types;
pub mod xml;

pub use db::{
    create_search_index, get_all_archives, get_ignored_archives, is_search_index_valid,
    write_books_to_db, write_books_to_db_conn,
};
pub use events::{IndexingInfo, IndexingProgress, IndexingWarning};
pub use extract_fields::{
    extract_authors, extract_date, extract_genres, extract_lang, extract_sequence, extract_title,
};
pub use parsing::{deserialize_fb2, parse_book_from_bytes, parse_zip_archive};
pub use types::{
    ArchiveInfo, ArchiveStatus, Author, Book, IndexState, IndexingMode, IndexingPhase,
};

use anyhow::{Context, Result};
use duckdb::{AccessMode, Connection, params};
use log::{info, warn};
use rayon::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::{fs, path::Path};

use crate::common::{create_database_connection, db_config, get_db_path};

fn check_cancelled(cancelled: &AtomicBool) -> Result<()> {
    anyhow::ensure!(!cancelled.load(Ordering::Relaxed), "Indexing cancelled");
    Ok(())
}

fn count_books_in_zips(
    zip_paths: &[std::path::PathBuf],
    on_progress: &(impl Fn(IndexingPhase, usize, usize) + Send + Sync),
    cancelled: &AtomicBool,
) -> Result<usize> {
    let total_zips = zip_paths.len();
    on_progress(IndexingPhase::Counting, 0, total_zips);

    let mut total_books = 0usize;
    for (i, zip_path) in zip_paths.iter().enumerate() {
        check_cancelled(cancelled)?;
        if let Ok(file) = fs::File::open(zip_path)
            && let Ok(archive) = zip::ZipArchive::new(file)
        {
            let fb2_count = (0..archive.len())
                .filter(|&i| {
                    archive.name_for_index(i).is_some_and(|name| {
                        Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("fb2"))
                    })
                })
                .count();
            total_books += fb2_count;
        }
        on_progress(IndexingPhase::Counting, i + 1, total_zips);
    }

    Ok(total_books)
}

pub fn index_library<F, W, I>(
    library_path: &Path,
    mode: &IndexingMode,
    on_progress: F,
    on_warning: W,
    on_info: I,
    cancelled: &AtomicBool,
) -> Result<()>
where
    F: Fn(IndexingPhase, usize, usize) + Send + Sync,
    W: Fn(&str) + Send + Sync,
    I: Fn(&str) + Send + Sync,
{
    crate::preflight::ensure_writable(library_path)?;

    let db_path = get_db_path(library_path);
    drop(create_database_connection(&db_path)?);

    info!("Starting indexing library at {}", library_path.display());

    if *mode == IndexingMode::Search {
        return rebuild_search_index(&db_path, &on_progress, cancelled);
    }

    let ignored_archives: HashSet<String> = match &mode {
        IndexingMode::Full => HashSet::new(),
        IndexingMode::New => get_ignored_archives(&db_path)
            .context("Failed to read indexed archives from database")?,
        IndexingMode::Archives(selected) => {
            let selected_set: HashSet<&str> = selected.iter().map(String::as_str).collect();
            let all_archives =
                get_all_archives(&db_path).context("Failed to read all archives from database")?;
            all_archives
                .into_iter()
                .filter(|a| !selected_set.contains(a.as_str()))
                .collect()
        }
        IndexingMode::Search => unreachable!(),
    };

    let zip_paths = collect_zip_paths(library_path, &ignored_archives)?;

    let total_books = count_books_in_zips(&zip_paths, &on_progress, cancelled)?;
    check_cancelled(cancelled)?;

    let books_completed = AtomicUsize::new(0);
    let archives_written = AtomicUsize::new(0);
    let total_zip_files = zip_paths.len();
    on_progress(IndexingPhase::Parsing, 0, total_books);

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(calculate_thread_count())
        .build()
        .context("Failed to create thread pool")?;

    let (tx, rx) = std::sync::mpsc::sync_channel::<(String, Vec<Book>)>(2);

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadWrite)?)
        .context("Failed to open database for writing books")?;
    conn.execute_batch("SET threads=1; SET preserve_insertion_order=false")
        .context("Failed to tune writer connection")?;

    let writer_cancelled = cancelled;
    let writer_warning = &on_warning;
    let writer_progress = &on_progress;

    std::thread::scope(|scope| -> Result<()> {
        let writer = scope.spawn(move || -> Result<()> {
            for (zip_filename, books) in rx {
                if writer_cancelled.load(Ordering::Relaxed) {
                    break;
                }
                if let Err(e) = write_books_to_db_conn(&conn, &zip_filename, &books) {
                    let msg = format!("Failed to write books from {zip_filename}: {e}");
                    log::warn!("{msg}");
                    writer_warning(&msg);
                }
                let written = archives_written.fetch_add(1, Ordering::Relaxed) + 1;
                writer_progress(IndexingPhase::Writing, written, total_zip_files);
            }
            check_cancelled(writer_cancelled)?;
            if archives_written.load(Ordering::Relaxed) > 0 {
                create_search_index(&conn, writer_progress, writer_cancelled)?;
            }
            Ok(())
        });

        pool.install(|| {
            zip_paths.par_iter().for_each(|zip_path| {
                if let Some(result) = parse_zip_archive(
                    zip_path,
                    &books_completed,
                    total_books,
                    &on_progress,
                    &on_warning,
                    &on_info,
                    cancelled,
                ) {
                    let _ = tx.send(result);
                }
            });
        });
        drop(tx);

        writer
            .join()
            .map_err(|_| anyhow::anyhow!("Writer thread panicked"))?
    })?;

    log::info!("Indexing complete.");

    Ok(())
}

fn collect_zip_paths(
    library_path: &Path,
    ignored_archives: &HashSet<String>,
) -> Result<Vec<std::path::PathBuf>> {
    Ok(fs::read_dir(library_path)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "zip"))
        .filter(|path| {
            let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
                warn!(
                    "Unable to get file name from path {}. Skipping.",
                    path.display()
                );
                return false;
            };
            if ignored_archives.contains(filename) {
                info!("Skipping archive: {filename}");
                return false;
            }
            true
        })
        .collect())
}

fn rebuild_search_index(
    db_path: &Path,
    on_progress: &(impl Fn(IndexingPhase, usize, usize) + Send + Sync),
    cancelled: &AtomicBool,
) -> Result<()> {
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite)?)
        .context("Failed to open database for search index rebuild")?;
    conn.execute_batch("SET threads=1")
        .context("Failed to tune connection")?;
    create_search_index(&conn, on_progress, cancelled)
}

pub fn check_index_state(library_path: &Path) -> Result<IndexState> {
    let db_path = get_db_path(library_path);

    if !db_path.exists() {
        let archives_new = count_zip_files(library_path);
        return Ok(IndexState {
            archives_indexed: 0,
            archives_pending: 0,
            archives_new,
            search_index_valid: true,
            total_books: 0,
        });
    }

    let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)
        .context("Failed to open database for index state check")?;

    let archives_indexed: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM archives WHERE status = ?",
            params![ArchiveStatus::Indexed],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let archives_pending: usize = conn
        .query_row(
            "SELECT COUNT(*) FROM archives WHERE status = ?",
            params![ArchiveStatus::Indexing],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let db_archive_names: HashSet<String> = conn
        .prepare("SELECT name FROM archives")?
        .query_map([], |row| row.get(0))?
        .filter_map(std::result::Result::ok)
        .collect();

    let disk_zip_count = count_zip_files(library_path);
    let archives_new = disk_zip_count.saturating_sub(db_archive_names.len());

    let total_books: usize = conn
        .query_row("SELECT COUNT(*) FROM books", [], |row| row.get(0))
        .unwrap_or(0);

    let search_index_valid = is_search_index_valid(&conn, total_books);

    Ok(IndexState {
        archives_indexed,
        archives_pending,
        archives_new,
        search_index_valid,
        total_books,
    })
}

pub fn list_archives(library_path: &Path) -> Result<Vec<ArchiveInfo>> {
    let db_path = get_db_path(library_path);

    let db_archives: Vec<ArchiveInfo> = if db_path.exists() {
        let conn = Connection::open_with_flags(&db_path, db_config(AccessMode::ReadOnly)?)
            .context("Failed to open database for archive listing")?;
        conn.prepare("SELECT name, status::VARCHAR FROM archives ORDER BY name")?
            .query_map([], |row| {
                Ok(ArchiveInfo {
                    name: row.get(0)?,
                    status: row.get(1)?,
                })
            })?
            .filter_map(std::result::Result::ok)
            .collect()
    } else {
        Vec::new()
    };

    let known_names: HashSet<String> = db_archives.iter().map(|a| a.name.clone()).collect();

    let mut archives = db_archives;
    if let Ok(entries) = fs::read_dir(library_path) {
        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "zip")
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && !known_names.contains(name)
            {
                archives.push(ArchiveInfo {
                    name: name.to_string(),
                    status: "new".to_string(),
                });
            }
        }
    }

    archives.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(archives)
}

fn count_zip_files(library_path: &Path) -> usize {
    fs::read_dir(library_path).map_or(0, |entries| {
        entries
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "zip"))
            .count()
    })
}

fn calculate_thread_count() -> usize {
    let cpus = std::thread::available_parallelism().map_or(4, std::num::NonZero::get);
    cpus.saturating_sub(1).max(2)
}
