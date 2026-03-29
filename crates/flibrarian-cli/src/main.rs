use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cli_table::Table;
use env_logger::Builder;
use log::LevelFilter;

use flibrarian_core::extracting::ExtractedBook;
use flibrarian_core::searching::FoundBook;

mod commands;

#[derive(Parser, Debug)]
#[command(
    name = "flibrarian",
    version,
    propagate_version = true,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, short, action = clap::ArgAction::Count, help = "Output verbosity: error, warn, info, debug, trace", global = true)]
    verbose: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Index the library
    Index {
        #[arg(required = true)]
        library_path: String,
        #[arg(
            long = "target",
            short = 't',
            value_name = "TARGET",
            default_value = "new",
            help = "Indexing target: 'full' for all archives, 'new' for new archives only, or a specific archive name"
        )]
        target: String,
    },
    /// Search the library
    Search {
        #[arg(required = true)]
        library_path: String,
        query: Option<String>,
        #[arg(long = "filter-title", short = 't')]
        filter_title: Option<String>,
        #[arg(long = "filter-authors", short = 'a')]
        filter_authors: Option<String>,
        #[arg(long = "filter-genres", short = 'g')]
        filter_genres: Option<String>,
        #[arg(long = "filter-date", short = 'd')]
        filter_date: Option<String>,
        #[arg(long = "filter-lang", short = 'l')]
        filter_lang: Option<String>,
        #[arg(long = "filter-sequence", short = 's')]
        filter_sequence: Option<String>,
    },
    /// Extract books by their IDs
    Extract {
        #[arg(required = true)]
        library_path: String,
        #[arg(required = true)]
        book_ids: Vec<u32>,
        #[arg(
            long = "output-dir",
            short = 'o',
            value_name = "DIR",
            help = "Output directory (defaults to settings value, then CWD)"
        )]
        output_dir: Option<String>,
    },
    /// Generate fake book archives for benchmarking
    Fake {
        #[arg(required = true)]
        output_dir: String,
        #[arg(long = "archives", short = 'n', default_value = "1")]
        archives: u32,
        #[arg(long = "books", short = 'b', default_value = "100")]
        books_per_archive: u32,
        #[arg(long = "seed", short = 's')]
        seed: Option<u64>,
        #[arg(long = "lang", short = 'l', default_value = "en")]
        lang: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    Builder::new()
        .filter_level(match cli.verbose {
            0 => LevelFilter::Error,
            1 => LevelFilter::Warn,
            2 => LevelFilter::Info,
            3 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init();

    match cli.command {
        Commands::Index {
            library_path,
            target,
        } => commands::run_index(&library_path, &target)?,
        Commands::Search {
            library_path,
            query,
            filter_title,
            filter_authors,
            filter_genres,
            filter_date,
            filter_lang,
            filter_sequence,
        } => commands::run_search(
            &library_path,
            query,
            filter_title,
            filter_authors,
            filter_genres,
            filter_date,
            filter_lang,
            filter_sequence,
        )?,
        Commands::Extract {
            library_path,
            book_ids,
            output_dir,
        } => commands::run_extract(&library_path, &book_ids, output_dir)?,
        Commands::Fake {
            output_dir,
            archives,
            books_per_archive,
            seed,
            lang,
        } => commands::run_fake(&output_dir, archives, books_per_archive, seed, &lang)?,
    }

    Ok(())
}

#[allow(clippy::cast_precision_loss)]
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn print_search_results(results: &[FoundBook]) -> Result<()> {
    if results.is_empty() {
        println!("No results found.");
        return Ok(());
    }

    let table = results
        .iter()
        .map(|r| {
            let authors = r
                .authors
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ");

            let genres = r.genres.join(", ");

            vec![
                r.id.to_string(),
                format!("{:.80}", r.title),
                format!("{:.70}", authors),
                format!("{:.40}", genres),
                format!("{:.20}", r.date),
                r.lang.clone(),
                format_file_size(r.file_size),
                format!("{:.60}", r.sequence),
                format!("{:.2}", r.score),
            ]
        })
        .collect::<Vec<_>>()
        .table()
        .title([
            "ID", "Title", "Authors", "Genres", "Date", "Lang", "Size", "Sequence", "Score",
        ]);

    cli_table::print_stdout(table).context("Failed to print search results table")?;

    Ok(())
}

fn print_extraction_results(results: &[ExtractedBook]) {
    if results.is_empty() {
        println!("No books were extracted.");
        return;
    }

    println!("Extracted {} book(s):", results.len());
    for book in results {
        println!("  {} — {} [{}]", book.author, book.title, book.id);
    }
}
