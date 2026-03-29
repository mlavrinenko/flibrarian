use super::types::{Author, PartialDescription, PartialTitleInfo};
use super::xml::convert_fb2_author_to_struct;

#[cfg(test)]
#[path = "extract_fields_tests.rs"]
mod tests;

#[must_use]
pub fn extract_title(title_info: &PartialTitleInfo) -> String {
    title_info
        .book_title
        .as_ref()
        .map(|titles| {
            titles
                .iter()
                .map(|title| title.value.clone())
                .collect::<Vec<String>>()
                .join(" ")
        })
        .unwrap_or_default()
}

#[must_use]
pub fn extract_date(title_info: &PartialTitleInfo, description: &PartialDescription) -> String {
    let from_title = title_info
        .date
        .as_ref()
        .and_then(|dates| dates.last().map(|date| date.value.clone()))
        .unwrap_or_default();

    if !from_title.is_empty() {
        return from_title;
    }

    let from_publish = description
        .publish_info
        .as_ref()
        .and_then(|pubs| {
            pubs.first().and_then(|pi| {
                pi.year
                    .as_ref()
                    .and_then(|years| years.last().map(|y| y.value.clone()))
            })
        })
        .unwrap_or_default();

    if !from_publish.is_empty() {
        return from_publish;
    }

    description
        .document_info
        .as_ref()
        .and_then(|docs| {
            docs.first().and_then(|di| {
                di.date
                    .as_ref()
                    .and_then(|dates| dates.last().map(|d| d.value.clone()))
            })
        })
        .unwrap_or_default()
}

#[must_use]
pub fn extract_sequence(title_info: &PartialTitleInfo) -> String {
    title_info
        .sequence
        .as_ref()
        .map(|seqs| {
            seqs.iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<String>>()
                .join(", ")
        })
        .unwrap_or_default()
}

#[must_use]
pub fn extract_lang(title_info: &PartialTitleInfo) -> String {
    title_info
        .lang
        .as_ref()
        .and_then(|langs| langs.first().cloned())
        .unwrap_or_default()
}

#[must_use]
pub fn extract_genres(title_info: &PartialTitleInfo) -> Vec<String> {
    let mut genres = title_info.genres.clone().unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    genres.retain(|g| seen.insert(g.clone()));
    genres
}

#[must_use]
pub fn extract_authors(title_info: &PartialTitleInfo) -> Vec<Author> {
    title_info
        .authors
        .iter()
        .flat_map(|author| convert_fb2_author_to_struct(author.clone()))
        .collect()
}
