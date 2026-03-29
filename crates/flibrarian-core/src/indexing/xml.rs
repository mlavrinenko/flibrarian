use memchr::memmem;
use regex::Regex;
use std::sync::LazyLock;
use uuid::Uuid;

use super::types::{Author, PartialFb2Author};

static XML_DECL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^<\?xml[^>]*\?>").unwrap());
static FICTIONBOOK_OPEN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^<FictionBook[^>]*>").unwrap());
static DESCRIPTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<description[^>]*>.*?</description>").unwrap());

static DESC_OPEN_FINDER: LazyLock<memmem::Finder> =
    LazyLock::new(|| memmem::Finder::new(b"<description"));
static DESC_CLOSE_FINDER: LazyLock<memmem::Finder> =
    LazyLock::new(|| memmem::Finder::new(b"</description>"));

pub fn extract_xml_declaration_and_fictionbook_tag(raw_bytes: &[u8]) -> (String, String) {
    let sample_size = std::cmp::min(raw_bytes.len(), 1024);
    let sample_content = String::from_utf8_lossy(&raw_bytes[..sample_size]);

    let xml_decl = XML_DECL_REGEX.find(&sample_content).map_or_else(
        || r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string(),
        |m| m.as_str().to_string(),
    );

    let fictionbook_open = FICTIONBOOK_OPEN_REGEX
        .find(&sample_content).map_or_else(|| r#"<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0" xmlns:l="http://www.w3.org/1999/xlink">"#.to_string(), |m| m.as_str().to_string());

    (xml_decl, fictionbook_open)
}

pub(crate) fn extract_description_bytes(raw_bytes: &[u8]) -> Option<&[u8]> {
    let start = DESC_OPEN_FINDER.find(raw_bytes)?;
    let after_start = start + b"<description".len();
    let close_pos = DESC_CLOSE_FINDER.find(&raw_bytes[after_start..])?;
    let end = after_start + close_pos + b"</description>".len();
    Some(&raw_bytes[start..end])
}

pub fn create_template_with_description(
    utf8_content: &str,
    xml_decl: &str,
    fictionbook_open: &str,
) -> String {
    let description = DESCRIPTION_REGEX.find(utf8_content).map_or_else(
        || {
            "<description><title-info><book-title>Unknown</book-title></title-info></description>"
                .to_string()
        },
        |m| m.as_str().to_string(),
    );

    format!("{xml_decl}\n{fictionbook_open}\n{description}\n<body></body>\n</FictionBook>")
}

pub(crate) fn build_template(xml_decl: &str, fictionbook_open: &str, description: &str) -> String {
    format!("{xml_decl}\n{fictionbook_open}\n{description}\n<body></body>\n</FictionBook>")
}

#[cfg(test)]
#[path = "xml_tests.rs"]
mod tests;

#[must_use]
pub fn convert_fb2_author_to_struct(author: PartialFb2Author) -> Vec<Author> {
    let first_names: Vec<String> = author
        .first_name
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.value.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    let middle_names: Vec<String> = author
        .middle_name
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.value.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    let last_names: Vec<String> = author
        .last_name
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.value.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();
    let nicknames: Vec<String> = author
        .nickname
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.value.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect();

    let max_count = [
        first_names.len(),
        middle_names.len(),
        last_names.len(),
        nicknames.len(),
    ]
    .into_iter()
    .max()
    .unwrap_or(0);

    if max_count == 0 {
        return vec![Author {
            id: author.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            first_name: None,
            middle_name: None,
            last_name: None,
            nickname: None,
        }];
    }

    let mut authors = Vec::new();

    for i in 0..max_count {
        let new_author = Author {
            id: author
                .id
                .clone()
                .unwrap_or_else(|| Uuid::new_v4().to_string()),
            first_name: first_names.get(i).cloned(),
            middle_name: middle_names.get(i).cloned(),
            last_name: last_names.get(i).cloned(),
            nickname: nicknames.get(i).cloned(),
        };

        if new_author.first_name.is_some()
            || new_author.middle_name.is_some()
            || new_author.last_name.is_some()
            || new_author.nickname.is_some()
        {
            authors.push(new_author);
        }
    }

    if authors.is_empty() {
        authors.push(Author {
            id: author.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            first_name: None,
            middle_name: None,
            last_name: None,
            nickname: None,
        });
    }

    authors
}
