pub mod fb2;
mod names;

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Context, Result};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use zip::write::SimpleFileOptions;

use fb2::generate_fb2_xml;

pub struct FakeLibraryConfig {
    pub output_dir: PathBuf,
    pub num_archives: u32,
    pub books_per_archive: u32,
    pub seed: Option<u64>,
    pub lang: String,
}

struct ArchiveTask {
    path: PathBuf,
    seed: u64,
    start_book_id: u32,
}

pub fn generate_fake_library<F>(config: &FakeLibraryConfig, on_progress: F) -> Result<()>
where
    F: Fn(usize, usize) + Send + Sync,
{
    std::fs::create_dir_all(&config.output_dir).context("Failed to create output directory")?;

    let tasks = build_archive_tasks(config);
    let total_books = usize::try_from(config.num_archives).unwrap_or(usize::MAX)
        * usize::try_from(config.books_per_archive).unwrap_or(usize::MAX);
    let progress = AtomicUsize::new(0);

    tasks.into_par_iter().try_for_each(|task| {
        let mut rng = StdRng::seed_from_u64(task.seed);
        write_archive(
            &task.path,
            config.books_per_archive,
            task.start_book_id,
            &config.lang,
            &mut rng,
            &progress,
            total_books,
            &on_progress,
        )
    })
}

fn build_archive_tasks(config: &FakeLibraryConfig) -> Vec<ArchiveTask> {
    let mut master_rng = config
        .seed
        .map_or_else(StdRng::from_entropy, StdRng::seed_from_u64);

    (0..config.num_archives)
        .map(|idx| {
            let seed: u64 = master_rng.r#gen();
            let start_book_id = idx * config.books_per_archive + 1;
            let path = config.output_dir.join(format!("fb2-fake-{idx:04}.zip"));
            ArchiveTask {
                path,
                seed,
                start_book_id,
            }
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn write_archive(
    path: &PathBuf,
    books_per_archive: u32,
    start_book_id: u32,
    lang: &str,
    rng: &mut StdRng,
    progress: &AtomicUsize,
    total: usize,
    on_progress: &(impl Fn(usize, usize) + Sync),
) -> Result<()> {
    let file =
        std::fs::File::create(path).context(format!("Failed to create {}", path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    for i in 0..books_per_archive {
        let book_id = start_book_id + i;
        let fb2_content = generate_fb2_xml(rng, lang);
        let filename = format!("{book_id}.fb2");

        zip.start_file(&filename, options)
            .context(format!("Failed to start file {filename} in archive"))?;
        zip.write_all(fb2_content.as_bytes())
            .context(format!("Failed to write {filename}"))?;

        let current = progress.fetch_add(1, Ordering::Relaxed) + 1;
        on_progress(current, total);
    }

    zip.finish().context("Failed to finalize zip archive")?;
    Ok(())
}
