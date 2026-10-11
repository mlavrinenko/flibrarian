use fb2::LocalizedText;
use flibrarian_core::indexing::types::{
    PartialDescription, PartialDocumentInfo, PartialFb2Author, PartialPublishInfo,
    PartialTitleInfo, Sequence,
};
use flibrarian_core::indexing::{
    deserialize_fb2, extract_authors, extract_date, extract_genres, extract_sequence,
    extract_title, parse_book_from_bytes,
};

fn localized(value: &str) -> LocalizedText {
    LocalizedText {
        lang: None,
        value: value.to_string(),
    }
}

fn make_title_info(title: &str) -> PartialTitleInfo {
    PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: Some(vec![localized(title)]),
        date: None,
        sequence: None,
        lang: None,
    }
}

#[test]
fn test_extract_title_single() {
    let ti = make_title_info("My Book");
    assert_eq!(extract_title(&ti), "My Book");
}

#[test]
fn test_extract_title_multiple_titles_joined() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: Some(vec![localized("Part One"), localized("Part Two")]),
        date: None,
        sequence: None,
        lang: None,
    };
    assert_eq!(extract_title(&ti), "Part One Part Two");
}

#[test]
fn test_extract_title_none() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: None,
        date: None,
        sequence: None,
        lang: None,
    };
    assert_eq!(extract_title(&ti), "");
}

#[test]
fn test_extract_title_empty_vec() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: Some(vec![]),
        date: None,
        sequence: None,
        lang: None,
    };
    assert_eq!(extract_title(&ti), "");
}

#[test]
fn test_extract_date_from_title_info() {
    let desc = PartialDescription {
        title_info: vec![PartialTitleInfo {
            genres: None,
            authors: vec![],
            book_title: Some(vec![localized("Book")]),
            date: Some(vec![localized("2023")]),
            sequence: None,
            lang: None,
        }],
        publish_info: None,
        document_info: None,
    };
    assert_eq!(extract_date(&desc.title_info[0], &desc), "2023");
}

#[test]
fn test_extract_date_fallback_to_publish_info() {
    let desc = PartialDescription {
        title_info: vec![PartialTitleInfo {
            genres: None,
            authors: vec![],
            book_title: Some(vec![localized("Book")]),
            date: None,
            sequence: None,
            lang: None,
        }],
        publish_info: Some(vec![PartialPublishInfo {
            year: Some(vec![localized("2020")]),
        }]),
        document_info: None,
    };
    assert_eq!(extract_date(&desc.title_info[0], &desc), "2020");
}

#[test]
fn test_extract_date_fallback_to_document_info() {
    let desc = PartialDescription {
        title_info: vec![PartialTitleInfo {
            genres: None,
            authors: vec![],
            book_title: Some(vec![localized("Book")]),
            date: None,
            sequence: None,
            lang: None,
        }],
        publish_info: None,
        document_info: Some(vec![PartialDocumentInfo {
            date: Some(vec![localized("2019")]),
        }]),
    };
    assert_eq!(extract_date(&desc.title_info[0], &desc), "2019");
}

#[test]
fn test_extract_date_empty_when_none_available() {
    let desc = PartialDescription {
        title_info: vec![make_title_info("Book")],
        publish_info: None,
        document_info: None,
    };
    assert_eq!(extract_date(&desc.title_info[0], &desc), "");
}

#[test]
fn test_extract_date_empty_title_date_falls_through() {
    let desc = PartialDescription {
        title_info: vec![PartialTitleInfo {
            genres: None,
            authors: vec![],
            book_title: Some(vec![localized("Book")]),
            date: Some(vec![localized("")]),
            sequence: None,
            lang: None,
        }],
        publish_info: Some(vec![PartialPublishInfo {
            year: Some(vec![localized("2020")]),
        }]),
        document_info: None,
    };
    assert_eq!(extract_date(&desc.title_info[0], &desc), "2020");
}

#[test]
fn test_extract_date_takes_last_from_title() {
    let desc = PartialDescription {
        title_info: vec![PartialTitleInfo {
            genres: None,
            authors: vec![],
            book_title: Some(vec![localized("Book")]),
            date: Some(vec![localized("2021"), localized("2023")]),
            sequence: None,
            lang: None,
        }],
        publish_info: None,
        document_info: None,
    };
    assert_eq!(extract_date(&desc.title_info[0], &desc), "2023");
}

#[test]
fn test_extract_sequence_single() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: Some(vec![localized("Book")]),
        date: None,
        sequence: Some(vec![Sequence {
            name: Some("Series A".to_string()),
            number: Some("3".to_string()),
            nested: None,
        }]),
        lang: None,
    };
    assert_eq!(extract_sequence(&ti), "Series A 3");
}

#[test]
fn test_extract_sequence_multiple() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: Some(vec![localized("Book")]),
        date: None,
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
        lang: None,
    };
    assert_eq!(extract_sequence(&ti), "Series A 1, Series B 2");
}

#[test]
fn test_extract_sequence_none() {
    let ti = make_title_info("Book");
    assert_eq!(extract_sequence(&ti), "");
}

#[test]
fn test_extract_genres_some() {
    let ti = PartialTitleInfo {
        genres: Some(vec!["sci_fi".to_string(), "adventure".to_string()]),
        authors: vec![],
        book_title: Some(vec![localized("Book")]),
        date: None,
        sequence: None,
        lang: None,
    };
    assert_eq!(extract_genres(&ti), vec!["sci_fi", "adventure"]);
}

#[test]
fn test_extract_genres_none() {
    let ti = make_title_info("Book");
    assert!(extract_genres(&ti).is_empty());
}

#[test]
fn test_extract_genres_deduplicates() {
    let ti = PartialTitleInfo {
        genres: Some(vec![
            "sci_fi".to_string(),
            "adventure".to_string(),
            "sci_fi".to_string(),
        ]),
        authors: vec![],
        book_title: Some(vec![localized("Book")]),
        date: None,
        sequence: None,
        lang: None,
    };
    assert_eq!(extract_genres(&ti), vec!["sci_fi", "adventure"]);
}

#[test]
fn test_extract_authors_with_author() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![PartialFb2Author {
            first_name: Some(vec![localized("Ivan")]),
            middle_name: None,
            last_name: Some(vec![localized("Petrov")]),
            nickname: None,
            id: Some("author-1".to_string()),
        }],
        book_title: Some(vec![localized("Book")]),
        date: None,
        sequence: None,
        lang: None,
    };
    let authors = extract_authors(&ti);
    assert_eq!(authors.len(), 1);
    assert_eq!(authors[0].first_name, Some("Ivan".to_string()));
    assert_eq!(authors[0].last_name, Some("Petrov".to_string()));
}

#[test]
fn test_extract_authors_empty() {
    let ti = make_title_info("Book");
    assert!(extract_authors(&ti).is_empty());
}

#[test]
fn test_deserialize_fb2_minimal() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>Test Book</book-title>
<author><first-name>Test</first-name></author>
</title-info>
</description>
<body><section><p>Hello</p></section></body>
</FictionBook>"#;

    let fb = deserialize_fb2(xml).unwrap();
    assert_eq!(fb.description.title_info.len(), 1);
    assert!(fb.description.title_info[0].has_book_title());
}

#[test]
fn test_deserialize_fb2_no_title_fails() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<author><first-name>Test</first-name></author>
</title-info>
</description>
<body></body>
</FictionBook>"#;

    assert!(deserialize_fb2(xml).is_err());
}

#[test]
fn test_parse_book_from_bytes_full() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<genre>fantasy</genre>
<genre>adventure</genre>
<author><first-name>Jane</first-name><last-name>Doe</last-name></author>
<book-title>Great Adventure</book-title>
<date>2023</date>
<sequence name="Epic Saga" number="1"/>
</title-info>
</description>
<body></body>
</FictionBook>"#;

    let book = parse_book_from_bytes(42, xml.as_ref()).unwrap();
    assert_eq!(book.id, 42);
    assert_eq!(book.title, "Great Adventure");
    assert_eq!(book.date, "2023");
    assert_eq!(book.genres, vec!["fantasy", "adventure"]);
    assert_eq!(book.sequence, "Epic Saga 1");
    assert_eq!(book.authors.len(), 1);
    assert_eq!(book.authors[0].first_name, Some("Jane".to_string()));
    assert_eq!(book.authors[0].last_name, Some("Doe".to_string()));
}

#[test]
fn test_parse_book_from_bytes_minimal() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>Minimal</book-title>
</title-info>
</description>
<body></body>
</FictionBook>"#;

    let book = parse_book_from_bytes(1, xml.as_ref()).unwrap();
    assert_eq!(book.title, "Minimal");
    assert!(book.genres.is_empty());
    assert_eq!(book.date, "");
    assert_eq!(book.sequence, "");
}

#[test]
fn test_has_book_title_true() {
    let ti = make_title_info("Title");
    assert!(ti.has_book_title());
}

#[test]
fn test_has_book_title_false_none() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: None,
        date: None,
        sequence: None,
        lang: None,
    };
    assert!(!ti.has_book_title());
}

#[test]
fn test_has_book_title_false_empty() {
    let ti = PartialTitleInfo {
        genres: None,
        authors: vec![],
        book_title: Some(vec![]),
        date: None,
        sequence: None,
        lang: None,
    };
    assert!(!ti.has_book_title());
}
