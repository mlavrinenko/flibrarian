use super::*;
use crate::indexing::types::{PartialDocumentInfo, PartialFb2Author, PartialPublishInfo, Sequence};
use fb2::LocalizedText;

fn lt(value: &str) -> LocalizedText {
    LocalizedText {
        lang: None,
        value: value.to_string(),
    }
}

fn empty_title_info() -> PartialTitleInfo {
    PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: None,
        date: None,
        sequence: None,
        lang: None,
    }
}

fn empty_description() -> PartialDescription {
    PartialDescription {
        title_info: vec![empty_title_info()],
        publish_info: None,
        document_info: None,
    }
}

#[test]
fn extract_title_single() {
    let ti = PartialTitleInfo {
        book_title: Some(vec![lt("My Book")]),
        ..empty_title_info()
    };
    assert_eq!(extract_title(&ti), "My Book");
}

#[test]
fn extract_title_multiple_joined() {
    let ti = PartialTitleInfo {
        book_title: Some(vec![lt("Part One"), lt("Part Two")]),
        ..empty_title_info()
    };
    assert_eq!(extract_title(&ti), "Part One Part Two");
}

#[test]
fn extract_title_missing() {
    assert_eq!(extract_title(&empty_title_info()), "");
}

#[test]
fn extract_title_empty_vec() {
    let ti = PartialTitleInfo {
        book_title: Some(vec![]),
        ..empty_title_info()
    };
    assert_eq!(extract_title(&ti), "");
}

#[test]
fn extract_date_from_title_info() {
    let ti = PartialTitleInfo {
        date: Some(vec![lt("2023")]),
        ..empty_title_info()
    };
    let desc = PartialDescription {
        title_info: vec![empty_title_info()],
        publish_info: Some(vec![PartialPublishInfo {
            year: Some(vec![lt("2020")]),
        }]),
        document_info: None,
    };
    assert_eq!(extract_date(&ti, &desc), "2023");
}

#[test]
fn extract_date_fallback_to_publish_info() {
    let ti = empty_title_info();
    let desc = PartialDescription {
        title_info: vec![empty_title_info()],
        publish_info: Some(vec![PartialPublishInfo {
            year: Some(vec![lt("2020")]),
        }]),
        document_info: Some(vec![PartialDocumentInfo {
            date: Some(vec![lt("2018")]),
        }]),
    };
    assert_eq!(extract_date(&ti, &desc), "2020");
}

#[test]
fn extract_date_fallback_to_document_info() {
    let ti = empty_title_info();
    let desc = PartialDescription {
        title_info: vec![empty_title_info()],
        publish_info: None,
        document_info: Some(vec![PartialDocumentInfo {
            date: Some(vec![lt("2018")]),
        }]),
    };
    assert_eq!(extract_date(&ti, &desc), "2018");
}

#[test]
fn extract_date_all_empty() {
    let ti = empty_title_info();
    let desc = empty_description();
    assert_eq!(extract_date(&ti, &desc), "");
}

#[test]
fn extract_date_takes_last_from_title_info() {
    let ti = PartialTitleInfo {
        date: Some(vec![lt("2020"), lt("2023")]),
        ..empty_title_info()
    };
    let desc = empty_description();
    assert_eq!(extract_date(&ti, &desc), "2023");
}

#[test]
fn extract_date_skips_empty_title_date() {
    let ti = PartialTitleInfo {
        date: Some(vec![lt("")]),
        ..empty_title_info()
    };
    let desc = PartialDescription {
        title_info: vec![empty_title_info()],
        publish_info: Some(vec![PartialPublishInfo {
            year: Some(vec![lt("2021")]),
        }]),
        document_info: None,
    };
    assert_eq!(extract_date(&ti, &desc), "2021");
}

#[test]
fn extract_lang_first_value() {
    let ti = PartialTitleInfo {
        lang: Some(vec!["en".to_string(), "ru".to_string()]),
        ..empty_title_info()
    };
    assert_eq!(extract_lang(&ti), "en");
}

#[test]
fn extract_lang_missing() {
    assert_eq!(extract_lang(&empty_title_info()), "");
}

#[test]
fn extract_lang_empty_vec() {
    let ti = PartialTitleInfo {
        lang: Some(vec![]),
        ..empty_title_info()
    };
    assert_eq!(extract_lang(&ti), "");
}

#[test]
fn extract_genres_deduplicates() {
    let ti = PartialTitleInfo {
        genres: Some(vec![
            "fantasy".to_string(),
            "sci_fi".to_string(),
            "fantasy".to_string(),
        ]),
        ..empty_title_info()
    };
    let result = extract_genres(&ti);
    assert_eq!(result, vec!["fantasy", "sci_fi"]);
}

#[test]
fn extract_genres_empty() {
    assert!(extract_genres(&empty_title_info()).is_empty());
}

#[test]
fn extract_genres_preserves_order() {
    let ti = PartialTitleInfo {
        genres: Some(vec!["c".to_string(), "a".to_string(), "b".to_string()]),
        ..empty_title_info()
    };
    assert_eq!(extract_genres(&ti), vec!["c", "a", "b"]);
}

#[test]
fn extract_sequence_single() {
    let ti = PartialTitleInfo {
        sequence: Some(vec![Sequence {
            name: Some("My Series".to_string()),
            number: Some("3".to_string()),
            nested: None,
        }]),
        ..empty_title_info()
    };
    assert_eq!(extract_sequence(&ti), "My Series 3");
}

#[test]
fn extract_sequence_multiple_joined() {
    let ti = PartialTitleInfo {
        sequence: Some(vec![
            Sequence {
                name: Some("Series A".to_string()),
                number: Some("1".to_string()),
                nested: None,
            },
            Sequence {
                name: Some("Series B".to_string()),
                number: Some("2".to_string()),
                nested: None,
            },
        ]),
        ..empty_title_info()
    };
    assert_eq!(extract_sequence(&ti), "Series A 1, Series B 2");
}

#[test]
fn extract_sequence_empty() {
    assert_eq!(extract_sequence(&empty_title_info()), "");
}

#[test]
fn extract_authors_from_title_info() {
    let ti = PartialTitleInfo {
        authors: vec![PartialFb2Author {
            first_name: Some(vec![lt("John")]),
            middle_name: None,
            last_name: Some(vec![lt("Doe")]),
            nickname: None,
            id: Some("a1".to_string()),
        }],
        ..empty_title_info()
    };
    let result = extract_authors(&ti);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].first_name, Some("John".to_string()));
    assert_eq!(result[0].last_name, Some("Doe".to_string()));
}

#[test]
fn extract_authors_empty() {
    assert!(extract_authors(&empty_title_info()).is_empty());
}

#[test]
fn extract_authors_multiple() {
    let ti = PartialTitleInfo {
        authors: vec![
            PartialFb2Author {
                first_name: Some(vec![lt("Alice")]),
                middle_name: None,
                last_name: None,
                nickname: None,
                id: Some("a1".to_string()),
            },
            PartialFb2Author {
                first_name: Some(vec![lt("Bob")]),
                middle_name: None,
                last_name: None,
                nickname: None,
                id: Some("a2".to_string()),
            },
        ],
        ..empty_title_info()
    };
    let result = extract_authors(&ti);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].first_name, Some("Alice".to_string()));
    assert_eq!(result[1].first_name, Some("Bob".to_string()));
}
