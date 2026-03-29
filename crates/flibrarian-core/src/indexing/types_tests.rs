use super::*;

#[test]
fn index_state_needs_resume_pending() {
    let state = IndexState {
        archives_indexed: 1,
        archives_pending: 1,
        archives_new: 0,
        search_index_valid: true,
        total_books: 10,
    };
    assert!(state.needs_resume());
}

#[test]
fn index_state_needs_resume_new() {
    let state = IndexState {
        archives_indexed: 1,
        archives_pending: 0,
        archives_new: 2,
        search_index_valid: true,
        total_books: 10,
    };
    assert!(state.needs_resume());
}

#[test]
fn index_state_no_resume_needed() {
    let state = IndexState {
        archives_indexed: 3,
        archives_pending: 0,
        archives_new: 0,
        search_index_valid: true,
        total_books: 50,
    };
    assert!(!state.needs_resume());
}

#[test]
fn parse_id_valid() {
    assert_eq!(Book::parse_id("12345.fb2").unwrap(), 12345);
}

#[test]
fn parse_id_invalid_extension() {
    assert!(Book::parse_id("12345.txt").is_err());
}

#[test]
fn parse_id_no_extension() {
    assert!(Book::parse_id("12345").is_err());
}

#[test]
fn parse_id_non_numeric() {
    assert!(Book::parse_id("abc.fb2").is_err());
}

#[test]
fn parse_id_empty() {
    assert!(Book::parse_id("").is_err());
}

#[test]
fn archive_status_roundtrip() {
    assert_eq!(
        ArchiveStatus::from_str(ArchiveStatus::Indexing.as_str()).unwrap(),
        ArchiveStatus::Indexing
    );
    assert_eq!(
        ArchiveStatus::from_str(ArchiveStatus::Indexed.as_str()).unwrap(),
        ArchiveStatus::Indexed
    );
}

#[test]
fn archive_status_from_str_invalid() {
    assert!(ArchiveStatus::from_str("unknown").is_err());
}

#[test]
fn has_book_title_true() {
    let ti = PartialTitleInfo {
        book_title: Some(vec![LocalizedText {
            lang: None,
            value: "Test".to_string(),
        }]),
        genres: None,
        authors: vec![],
        date: None,
        sequence: None,
        lang: None,
    };
    assert!(ti.has_book_title());
}

#[test]
fn has_book_title_empty_vec() {
    let ti = PartialTitleInfo {
        book_title: Some(vec![]),
        genres: None,
        authors: vec![],
        date: None,
        sequence: None,
        lang: None,
    };
    assert!(!ti.has_book_title());
}

#[test]
fn has_book_title_none() {
    let ti = PartialTitleInfo {
        book_title: None,
        genres: None,
        authors: vec![],
        date: None,
        sequence: None,
        lang: None,
    };
    assert!(!ti.has_book_title());
}

#[test]
fn author_display_all_none_anonymous() {
    let a = Author {
        id: "1".to_string(),
        first_name: None,
        middle_name: None,
        last_name: None,
        nickname: None,
    };
    assert_eq!(a.to_string(), "Anonymous");
}

#[test]
fn sequence_display_with_nested() {
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
fn sequence_display_no_name_no_number() {
    let seq = Sequence {
        name: None,
        number: None,
        nested: None,
    };
    assert_eq!(seq.to_string(), "");
}
