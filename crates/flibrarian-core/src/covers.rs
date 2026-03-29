use crate::common::{db_config, get_db_path};
use crate::encoding::bytes_to_utf8_string;
use anyhow::{Context, Result};
use base64::Engine;
use duckdb::{AccessMode, Connection, params};
use log::warn;
use regex::Regex;
use std::io::Read;
use std::path::Path;
use std::sync::LazyLock;
use zip::ZipArchive;

static COVERPAGE_HREF_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r##"(?s)<coverpage[^>]*>.*?href\s*=\s*"#([^"]+)".*?</coverpage>"##).unwrap()
});

static BINARY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)<binary[^>]+id\s*=\s*"(?P<id>[^"]+)"[^>]+content-type\s*=\s*"(?P<ct>[^"]+)"[^>]*>\s*(?P<data>[A-Za-z0-9+/\s=]+?)\s*</binary>"#,
    )
    .unwrap()
});

pub struct CoverImage {
    pub data: Vec<u8>,
    pub content_type: String,
}

pub fn get_book_cover(library_path: &Path, book_id: u32) -> Result<Option<CoverImage>> {
    let Some(archive_name) = find_archive_for_book(library_path, book_id)? else {
        return Ok(None);
    };

    let Some(raw_bytes) = read_fb2_from_archive(library_path, &archive_name, book_id)? else {
        return Ok(None);
    };

    let utf8_content = fb2_bytes_to_utf8(&raw_bytes)?;

    let Some(cover_id) = extract_cover_id(&utf8_content) else {
        return Ok(None);
    };

    Ok(extract_binary_by_id(&utf8_content, &cover_id))
}

pub(crate) fn find_archive_for_book(library_path: &Path, book_id: u32) -> Result<Option<String>> {
    let db_path = get_db_path(library_path);
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadOnly)?)
        .context("Failed to open database")?;

    let result: Option<String> = conn
        .query_row(
            "SELECT a.name FROM books b JOIN archives a ON b.archive_id = a.id WHERE b.id = ?",
            params![book_id],
            |row| row.get(0),
        )
        .ok();

    Ok(result)
}

pub(crate) fn read_fb2_from_archive(
    library_path: &Path,
    archive_name: &str,
    book_id: u32,
) -> Result<Option<Vec<u8>>> {
    let zip_path = library_path.join(archive_name);
    let file = match std::fs::File::open(&zip_path) {
        Ok(f) => f,
        Err(e) => {
            warn!("Failed to open archive {}: {e}", zip_path.display());
            return Ok(None);
        }
    };

    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            warn!("Failed to read archive {}: {e}", zip_path.display());
            return Ok(None);
        }
    };

    let entry_name = format!("{book_id}.fb2");
    let mut file = match archive.by_name(&entry_name) {
        Ok(f) => f,
        Err(e) => {
            warn!("Book {entry_name} not found in {archive_name}: {e}");
            return Ok(None);
        }
    };

    let mut raw_bytes = Vec::new();
    file.read_to_end(&mut raw_bytes)?;
    Ok(Some(raw_bytes))
}

pub(crate) fn fb2_bytes_to_utf8(raw_bytes: &[u8]) -> Result<String> {
    bytes_to_utf8_string(raw_bytes).context("Failed to convert bytes to UTF-8")
}

fn extract_cover_id(content: &str) -> Option<String> {
    COVERPAGE_HREF_REGEX
        .captures(content)
        .map(|caps| caps[1].to_string())
}

fn extract_binary_by_id(content: &str, cover_id: &str) -> Option<CoverImage> {
    for caps in BINARY_REGEX.captures_iter(content) {
        if &caps["id"] == cover_id {
            let content_type = caps["ct"].to_string();
            let raw_base64: String = caps["data"].split_whitespace().collect();

            let data = match base64::engine::general_purpose::STANDARD.decode(&raw_base64) {
                Ok(d) => d,
                Err(e) => {
                    warn!("Failed to decode base64 cover data for {cover_id}: {e}");
                    return None;
                }
            };

            return Some(CoverImage { data, content_type });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_cover_id_finds_id() {
        let content = r##"<coverpage><image l:href="#cover.jpeg"/></coverpage>"##;
        let result = extract_cover_id(content);
        assert_eq!(result, Some("cover.jpeg".to_string()));
    }

    #[test]
    fn extract_cover_id_no_coverpage() {
        let content = "<title-info><book-title>Test</book-title></title-info>";
        assert!(extract_cover_id(content).is_none());
    }

    #[test]
    fn extract_binary_by_id_finds_binary() {
        let content =
            r#"<binary id="cover.jpeg" content-type="image/jpeg">SGVsbG8gV29ybGQ=</binary>"#;
        let result = extract_binary_by_id(content, "cover.jpeg");
        assert!(result.is_some());
        let cover = result.unwrap();
        assert_eq!(cover.content_type, "image/jpeg");
        assert_eq!(cover.data, b"Hello World");
    }

    #[test]
    fn extract_binary_by_id_not_found() {
        let content = r#"<binary id="other.jpeg" content-type="image/jpeg">SGVsbG8=</binary>"#;
        assert!(extract_binary_by_id(content, "cover.jpeg").is_none());
    }

    #[test]
    fn extract_binary_by_id_invalid_base64() {
        let content =
            r#"<binary id="bad" content-type="image/jpeg">!!!not-valid-base64!!!</binary>"#;
        assert!(extract_binary_by_id(content, "bad").is_none());
    }

    #[test]
    fn extract_binary_by_id_multiline_base64() {
        let content = r#"<binary id="cover.png" content-type="image/png">
            SGVs
            bG8=
        </binary>"#;
        let result = extract_binary_by_id(content, "cover.png").unwrap();
        assert_eq!(result.content_type, "image/png");
        assert_eq!(result.data, b"Hello");
    }

    #[test]
    fn extract_binary_by_id_selects_correct_among_multiple() {
        let content = r#"
            <binary id="img1.jpeg" content-type="image/jpeg">AQID</binary>
            <binary id="img2.png" content-type="image/png">BAUG</binary>
        "#;
        let result = extract_binary_by_id(content, "img2.png").unwrap();
        assert_eq!(result.content_type, "image/png");
        assert_eq!(result.data, vec![4, 5, 6]);
    }

    #[test]
    fn extract_cover_id_with_multiline_coverpage() {
        let content = r##"<coverpage>
            <image l:href="#my_cover.jpg"/>
        </coverpage>"##;
        assert_eq!(extract_cover_id(content), Some("my_cover.jpg".to_string()));
    }
}
