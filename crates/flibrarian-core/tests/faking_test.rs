#[cfg(feature = "faking")]
mod tests {
    use flibrarian_core::faking::{FakeLibraryConfig, generate_fake_library};
    use flibrarian_core::indexing::parse_book_from_bytes;
    use flibrarian_core::indexing::types::Book;
    use std::io::Read;
    use tempfile::TempDir;

    #[test]
    fn generated_fb2_is_parseable() {
        let tmp = TempDir::new().unwrap();
        let config = FakeLibraryConfig {
            output_dir: tmp.path().to_path_buf(),
            num_archives: 1,
            books_per_archive: 5,
            seed: Some(42),
            lang: "en".to_string(),
        };

        generate_fake_library(&config, |_, _| {}).unwrap();

        let archive_path = tmp.path().join("fb2-fake-0000.zip");
        assert!(archive_path.exists());

        let file = std::fs::File::open(&archive_path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();

        assert_eq!(archive.len(), 5);

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).unwrap();
            let filename = entry.name().to_string();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).unwrap();
            let id = Book::parse_id(&filename).unwrap();
            let book = parse_book_from_bytes(id, &content);
            assert!(book.is_ok(), "Failed to parse {filename}: {book:?}");
        }
    }

    #[test]
    fn seeded_generation_is_deterministic() {
        let tmp1 = TempDir::new().unwrap();
        let tmp2 = TempDir::new().unwrap();

        let make_config = |path: std::path::PathBuf| FakeLibraryConfig {
            output_dir: path,
            num_archives: 1,
            books_per_archive: 3,
            seed: Some(123),
            lang: "en".to_string(),
        };

        generate_fake_library(&make_config(tmp1.path().to_path_buf()), |_, _| {}).unwrap();
        generate_fake_library(&make_config(tmp2.path().to_path_buf()), |_, _| {}).unwrap();

        let read_archive = |dir: &std::path::Path| -> Vec<Vec<u8>> {
            let file = std::fs::File::open(dir.join("fb2-fake-0000.zip")).unwrap();
            let mut archive = zip::ZipArchive::new(file).unwrap();
            (0..archive.len())
                .map(|i| {
                    let mut entry = archive.by_index(i).unwrap();
                    let mut buf = Vec::new();
                    entry.read_to_end(&mut buf).unwrap();
                    buf
                })
                .collect()
        };

        assert_eq!(read_archive(tmp1.path()), read_archive(tmp2.path()));
    }

    #[test]
    fn generates_multiple_archives() {
        let tmp = TempDir::new().unwrap();
        let config = FakeLibraryConfig {
            output_dir: tmp.path().to_path_buf(),
            num_archives: 3,
            books_per_archive: 2,
            seed: Some(99),
            lang: "ru".to_string(),
        };

        generate_fake_library(&config, |_, _| {}).unwrap();

        for i in 0..3u32 {
            let path = tmp.path().join(format!("fb2-fake-{i:04}.zip"));
            assert!(path.exists(), "Archive {i} should exist");
            let file = std::fs::File::open(&path).unwrap();
            let archive = zip::ZipArchive::new(file).unwrap();
            assert_eq!(archive.len(), 2);
        }
    }
}
