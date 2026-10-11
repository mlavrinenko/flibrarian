use duckdb::types::{FromSql, FromSqlError, ToSql, ValueRef};
use flibrarian_core::indexing::types::{
    ArchiveStatus, Author, Book, IndexingMode, PartialFictionBook, Sequence,
};
use std::str::FromStr;

#[test]
fn test_parse_id_valid() {
    assert_eq!(Book::parse_id("12345.fb2").unwrap(), 12345);
}

#[test]
fn test_parse_id_zero() {
    assert_eq!(Book::parse_id("0.fb2").unwrap(), 0);
}

#[test]
fn test_parse_id_large_number() {
    assert_eq!(Book::parse_id("4294967295.fb2").unwrap(), 4_294_967_295);
}

#[test]
fn test_parse_id_no_extension() {
    assert!(Book::parse_id("12345").is_err());
}

#[test]
fn test_parse_id_wrong_extension() {
    assert!(Book::parse_id("12345.txt").is_err());
}

#[test]
fn test_parse_id_non_numeric() {
    assert!(Book::parse_id("abc.fb2").is_err());
}

#[test]
fn test_parse_id_empty() {
    assert!(Book::parse_id("").is_err());
}

#[test]
fn test_parse_id_mixed() {
    assert!(Book::parse_id("123abc.fb2").is_err());
}

#[test]
fn test_author_display_full_name() {
    let author = Author {
        id: "1".to_string(),
        first_name: Some("John".to_string()),
        middle_name: Some("M".to_string()),
        last_name: Some("Doe".to_string()),
        nickname: None,
    };
    assert_eq!(author.to_string(), "John M Doe");
}

#[test]
fn test_author_display_first_last() {
    let author = Author {
        id: "1".to_string(),
        first_name: Some("John".to_string()),
        middle_name: None,
        last_name: Some("Doe".to_string()),
        nickname: None,
    };
    assert_eq!(author.to_string(), "John Doe");
}

#[test]
fn test_author_display_nickname_only() {
    let author = Author {
        id: "1".to_string(),
        first_name: None,
        middle_name: None,
        last_name: None,
        nickname: Some("TheNick".to_string()),
    };
    assert_eq!(author.to_string(), "TheNick");
}

#[test]
fn test_author_display_anonymous() {
    let author = Author {
        id: "1".to_string(),
        first_name: None,
        middle_name: None,
        last_name: None,
        nickname: None,
    };
    assert_eq!(author.to_string(), "Anonymous");
}

#[test]
fn test_author_display_empty_strings_treated_as_anonymous() {
    let author = Author {
        id: "1".to_string(),
        first_name: Some(String::new()),
        middle_name: Some(String::new()),
        last_name: Some(String::new()),
        nickname: Some(String::new()),
    };
    assert_eq!(author.to_string(), "Anonymous");
}

#[test]
fn test_sequence_display_name_and_number() {
    let seq = Sequence {
        name: Some("My Series".to_string()),
        number: Some("5".to_string()),
        nested: None,
    };
    assert_eq!(seq.to_string(), "My Series 5");
}

#[test]
fn test_sequence_display_name_only() {
    let seq = Sequence {
        name: Some("My Series".to_string()),
        number: None,
        nested: None,
    };
    assert_eq!(seq.to_string(), "My Series");
}

#[test]
fn test_sequence_display_number_only() {
    let seq = Sequence {
        name: None,
        number: Some("3".to_string()),
        nested: None,
    };
    assert_eq!(seq.to_string(), " 3");
}

#[test]
fn test_sequence_display_nested() {
    let seq = Sequence {
        name: Some("Parent".to_string()),
        number: Some("1".to_string()),
        nested: Some(vec![Sequence {
            name: Some("Child".to_string()),
            number: Some("2".to_string()),
            nested: None,
        }]),
    };
    assert_eq!(seq.to_string(), "Parent 1, Child 2");
}

#[test]
fn test_sequence_display_deeply_nested() {
    let seq = Sequence {
        name: Some("A".to_string()),
        number: None,
        nested: Some(vec![Sequence {
            name: Some("B".to_string()),
            number: None,
            nested: Some(vec![Sequence {
                name: Some("C".to_string()),
                number: Some("3".to_string()),
                nested: None,
            }]),
        }]),
    };
    assert_eq!(seq.to_string(), "A, B, C 3");
}

#[test]
fn test_archive_status_as_str() {
    assert_eq!(ArchiveStatus::Indexing.as_str(), "indexing");
    assert_eq!(ArchiveStatus::Indexed.as_str(), "indexed");
}

#[test]
fn test_archive_status_from_str() {
    assert_eq!(
        ArchiveStatus::from_str("indexing").unwrap(),
        ArchiveStatus::Indexing
    );
    assert_eq!(
        ArchiveStatus::from_str("indexed").unwrap(),
        ArchiveStatus::Indexed
    );
    assert!(ArchiveStatus::from_str("unknown").is_err());
}

#[test]
fn test_archive_status_roundtrip() {
    for status in [ArchiveStatus::Indexing, ArchiveStatus::Indexed] {
        assert_eq!(ArchiveStatus::from_str(status.as_str()).unwrap(), status);
    }
}

#[test]
fn test_indexing_mode_equality() {
    assert_eq!(IndexingMode::Full, IndexingMode::Full);
    assert_eq!(IndexingMode::New, IndexingMode::New);
    assert_eq!(
        IndexingMode::Archives(vec!["test.zip".to_string()]),
        IndexingMode::Archives(vec!["test.zip".to_string()])
    );
    assert_ne!(IndexingMode::Full, IndexingMode::New);
}

#[test]
fn test_archive_status_to_sql() {
    let indexing = ArchiveStatus::Indexing.to_sql().unwrap();
    let indexed = ArchiveStatus::Indexed.to_sql().unwrap();
    assert_eq!(
        format!("{indexing:?}"),
        format!("{:?}", duckdb::types::ToSqlOutput::from("indexing"))
    );
    assert_eq!(
        format!("{indexed:?}"),
        format!("{:?}", duckdb::types::ToSqlOutput::from("indexed"))
    );
}

#[test]
fn test_archive_status_from_sql_valid() {
    let value = duckdb::types::Value::Text("indexing".to_string());
    let vr = ValueRef::from(&value);
    let result = ArchiveStatus::column_result(vr).unwrap();
    assert_eq!(result, ArchiveStatus::Indexing);

    let value = duckdb::types::Value::Text("indexed".to_string());
    let vr = ValueRef::from(&value);
    let result = ArchiveStatus::column_result(vr).unwrap();
    assert_eq!(result, ArchiveStatus::Indexed);
}

#[test]
fn test_archive_status_from_sql_invalid_string() {
    let value = duckdb::types::Value::Text("garbage".to_string());
    let vr = ValueRef::from(&value);
    let result = ArchiveStatus::column_result(vr);
    assert!(matches!(result, Err(FromSqlError::InvalidType)));
}

#[test]
fn test_archive_status_from_sql_wrong_type() {
    let value = duckdb::types::Value::Int(42);
    let vr = ValueRef::from(&value);
    let result = ArchiveStatus::column_result(vr);
    assert!(matches!(result, Err(FromSqlError::InvalidType)));
}

#[test]
fn test_partial_description_deserialize_valid() {
    let xml = "<description>
        <title-info>
            <book-title>Test</book-title>
        </title-info>
    </description>";

    let desc: PartialFictionBook = quick_xml::de::from_str(&format!(
        r#"<?xml version="1.0"?><FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">{xml}<body></body></FictionBook>"#
    ))
    .unwrap();
    assert_eq!(desc.description.title_info.len(), 1);
    assert!(desc.description.title_info[0].has_book_title());
}

#[test]
fn test_partial_description_deserialize_no_book_title_fails() {
    let xml = r#"<?xml version="1.0"?><FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0"><description>
        <title-info>
            <author><first-name>NoTitle</first-name></author>
        </title-info>
    </description><body></body></FictionBook>"#;

    let result: Result<PartialFictionBook, _> = quick_xml::de::from_str(xml);
    assert!(result.is_err());
}

#[test]
fn test_sequence_display_empty() {
    let seq = Sequence {
        name: None,
        number: None,
        nested: None,
    };
    assert_eq!(seq.to_string(), "");
}

#[test]
fn test_sequence_serde_roundtrip() {
    let seq = Sequence {
        name: Some("Series".to_string()),
        number: Some("3".to_string()),
        nested: Some(vec![Sequence {
            name: Some("Sub".to_string()),
            number: None,
            nested: None,
        }]),
    };
    let json = serde_json::to_string(&seq).unwrap();
    let deser: Sequence = serde_json::from_str(&json).unwrap();
    assert_eq!(seq, deser);
}

#[test]
fn test_author_serde_roundtrip() {
    let author = Author {
        id: "test-id".to_string(),
        first_name: Some("John".to_string()),
        middle_name: None,
        last_name: Some("Doe".to_string()),
        nickname: None,
    };
    let json = serde_json::to_string(&author).unwrap();
    let deser: Author = serde_json::from_str(&json).unwrap();
    assert_eq!(author, deser);
}
