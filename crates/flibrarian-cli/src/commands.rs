use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use std::path::PathBuf;
use std::sync::Mutex;

use flibrarian_core::common::get_db_path;
use flibrarian_core::extracting;
use flibrarian_core::faking;
use flibrarian_core::indexing;
use flibrarian_core::searching;

use crate::{print_extraction_results, print_search_results};

pub fn run_index(library_path: &str, target: &str) -> Result<()> {
    let library_path: PathBuf = library_path.into();

    if !library_path.is_dir() {
        return Err(anyhow::anyhow!(
            "Library path '{}' does not exist or is not a directory.",
            library_path.display()
        ));
    }

    let mode = match target {
        "full" => indexing::IndexingMode::Full,
        "new" => indexing::IndexingMode::New,
        archive_name => indexing::IndexingMode::Archives(vec![archive_name.to_string()]),
    };

    let pb = Mutex::new(ProgressBar::new(0));

    indexing::index_library(
        &library_path,
        &mode,
        |phase, current, total| {
            let pb = pb.lock().unwrap();
            let current = u64::try_from(current).unwrap_or(u64::MAX);
            let total = u64::try_from(total).unwrap_or(u64::MAX);
            match phase {
                indexing::IndexingPhase::Counting | indexing::IndexingPhase::Writing => {
                    pb.set_style(bar_style("archives"));
                    pb.set_length(total);
                    pb.set_position(current);
                }
                indexing::IndexingPhase::Parsing => {
                    pb.set_style(bar_style("books"));
                    pb.set_length(total);
                    pb.set_position(current);
                }
                indexing::IndexingPhase::BuildingSearchIndex => {
                    pb.set_style(bar_style("search index"));
                    pb.set_length(total);
                    pb.set_position(current);
                }
                indexing::IndexingPhase::CreatingFtsIndex => {
                    pb.set_style(spinner_style());
                    pb.set_message("Creating FTS index...");
                    pb.enable_steady_tick(std::time::Duration::from_millis(100));
                }
            }
        },
        |warning| {
            log::warn!("{warning}");
        },
        |info_msg| {
            log::info!("{info_msg}");
        },
        &std::sync::atomic::AtomicBool::new(false),
    )?;
    pb.lock().unwrap().finish_and_clear();
    info!("Indexing completed successfully.");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn run_search(
    library_path: &str,
    query: Option<String>,
    filter_title: Option<String>,
    filter_authors: Option<String>,
    filter_genres: Option<String>,
    filter_date: Option<String>,
    filter_lang: Option<String>,
    filter_sequence: Option<String>,
) -> Result<()> {
    let query = query.unwrap_or_default();
    let filters = searching::SearchFilters {
        title: filter_title,
        authors: filter_authors,
        genres: filter_genres,
        date: filter_date,
        lang: filter_lang,
        file_size: None,
        sequence: filter_sequence,
    };

    if !has_any_filter(&filters) && query.trim().is_empty() {
        return Err(anyhow::anyhow!(
            "Please provide a search query or at least one filter (e.g. --filter-title, --filter-authors)."
        ));
    }

    let library_path: PathBuf = library_path.into();
    let db_path = get_db_path(&library_path);

    if !db_path.exists() {
        return Err(anyhow::anyhow!(
            "Database file not found at {}. Please run 'flibrarian index <library_path>' first.",
            db_path.display()
        ));
    }

    let results = searching::search_library(&library_path, &query, &filters)?;
    print_search_results(&results)?;
    info!("Search completed.");
    Ok(())
}

const fn has_any_filter(filters: &searching::SearchFilters) -> bool {
    filters.title.is_some()
        || filters.authors.is_some()
        || filters.genres.is_some()
        || filters.date.is_some()
        || filters.lang.is_some()
        || filters.sequence.is_some()
}

pub fn run_extract(library_path: &str, book_ids: &[u32], output_dir: Option<String>) -> Result<()> {
    let library_path: PathBuf = library_path.into();
    let output_dir = resolve_output_dir(output_dir)?;
    let results = extracting::extract_books(&library_path, book_ids, &output_dir)?;
    print_extraction_results(&results);
    info!("Extraction completed.");
    Ok(())
}

pub fn run_fake(
    output_dir: &str,
    archives: u32,
    books_per_archive: u32,
    seed: Option<u64>,
    lang: &str,
) -> Result<()> {
    let config = faking::FakeLibraryConfig {
        output_dir: output_dir.into(),
        num_archives: archives,
        books_per_archive,
        seed,
        lang: lang.to_string(),
    };

    let total = u64::from(archives) * u64::from(books_per_archive);
    let pb = ProgressBar::new(total);
    pb.set_style(bar_style("books"));

    faking::generate_fake_library(&config, |current, _total| {
        pb.set_position(u64::try_from(current).unwrap_or(u64::MAX));
    })?;

    pb.finish_and_clear();
    info!("Generated {archives} archive(s) with {books_per_archive} books each in {output_dir}");
    Ok(())
}

fn resolve_output_dir(explicit: Option<String>) -> Result<PathBuf> {
    use anyhow::Context;
    use flibrarian_core::settings;

    if let Some(dir) = explicit {
        return Ok(PathBuf::from(dir));
    }

    if let Some(dir) = settings::load_settings()
        .ok()
        .and_then(|s| s.default_save_folder)
    {
        return Ok(PathBuf::from(dir));
    }

    std::env::current_dir().context("Failed to get current working directory")
}

fn bar_style(unit: &str) -> ProgressStyle {
    ProgressStyle::with_template(&format!(
        "{{spinner:.green}} [{{bar:40.cyan/blue}}] {{pos}}/{{len}} {unit} ({{eta}})"
    ))
    .expect("valid progress bar template")
    .progress_chars("#>-")
}

fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.green} {msg}").expect("valid progress bar template")
}
