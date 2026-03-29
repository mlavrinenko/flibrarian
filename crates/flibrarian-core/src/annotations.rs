use anyhow::Result;
use regex::Regex;
use std::path::Path;
use std::sync::LazyLock;

use crate::covers::{fb2_bytes_to_utf8, find_archive_for_book, read_fb2_from_archive};

static ANNOTATION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<annotation[^>]*>(.*?)</annotation>").unwrap());

static P_CONTENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<p[^>]*>(.*?)</p>").unwrap());

static XML_TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());

pub fn get_book_annotation(library_path: &Path, book_id: u32) -> Result<Option<String>> {
    let Some(archive_name) = find_archive_for_book(library_path, book_id)? else {
        return Ok(None);
    };

    let Some(raw_bytes) = read_fb2_from_archive(library_path, &archive_name, book_id)? else {
        return Ok(None);
    };

    let utf8_content = fb2_bytes_to_utf8(&raw_bytes)?;

    Ok(extract_annotation(&utf8_content))
}

fn extract_annotation(content: &str) -> Option<String> {
    let annotation_block = ANNOTATION_REGEX.captures(content)?;
    let inner = &annotation_block[1];

    let paragraphs: Vec<String> = P_CONTENT_REGEX
        .captures_iter(inner)
        .map(|cap| strip_xml_tags(&cap[1]))
        .filter(|p| !p.is_empty())
        .collect();

    if paragraphs.is_empty() {
        let stripped = strip_xml_tags(inner).trim().to_string();
        if stripped.is_empty() {
            return None;
        }
        return Some(stripped);
    }

    Some(paragraphs.join("\n\n"))
}

fn strip_xml_tags(s: &str) -> String {
    XML_TAG_REGEX.replace_all(s, "").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_annotation_with_paragraphs() {
        let content = "<annotation><p>First paragraph</p><p>Second paragraph</p></annotation>";
        let result = extract_annotation(content).unwrap();
        assert_eq!(result, "First paragraph\n\nSecond paragraph");
    }

    #[test]
    fn extract_annotation_no_paragraphs_strips_tags() {
        let content = "<annotation><strong>Bold text</strong> and <em>italic</em></annotation>";
        let result = extract_annotation(content).unwrap();
        assert_eq!(result, "Bold text and italic");
    }

    #[test]
    fn extract_annotation_empty_returns_none() {
        let content = "<annotation></annotation>";
        assert!(extract_annotation(content).is_none());
    }

    #[test]
    fn extract_annotation_whitespace_only_returns_none() {
        let content = "<annotation>   </annotation>";
        assert!(extract_annotation(content).is_none());
    }

    #[test]
    fn extract_annotation_single_paragraph_no_p_tags() {
        let content = "<annotation>Just text</annotation>";
        let result = extract_annotation(content).unwrap();
        assert_eq!(result, "Just text");
    }

    #[test]
    fn extract_annotation_no_annotation_tag() {
        let content = "<title-info><book-title>Test</book-title></title-info>";
        assert!(extract_annotation(content).is_none());
    }

    #[test]
    fn extract_annotation_nested_tags_in_paragraphs() {
        let content =
            "<annotation><p>Text with <strong>bold</strong> and <a>link</a></p></annotation>";
        let result = extract_annotation(content).unwrap();
        assert_eq!(result, "Text with bold and link");
    }

    #[test]
    fn extract_annotation_empty_paragraphs_filtered() {
        let content = "<annotation><p></p><p>Real content</p><p>  </p></annotation>";
        let result = extract_annotation(content).unwrap();
        assert_eq!(result, "Real content");
    }

    #[test]
    fn strip_xml_tags_no_tags() {
        assert_eq!(strip_xml_tags("plain text"), "plain text");
    }

    #[test]
    fn strip_xml_tags_mixed() {
        assert_eq!(
            strip_xml_tags("<b>bold</b> and <i>italic</i>"),
            "bold and italic"
        );
    }
}
