use super::*;

#[test]
fn build_filter_conditions_empty() {
    let filters = SearchFilters::default();
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.is_empty());
    assert!(params.is_empty());
}

#[test]
fn build_filter_conditions_title_partial() {
    let filters = SearchFilters {
        title: Some("test".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert_eq!(sql, "AND b.title ILIKE ?");
    assert_eq!(params, vec!["%test%"]);
}

#[test]
fn build_filter_conditions_title_exact() {
    let filters = SearchFilters {
        title: Some("=exact title".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert_eq!(sql, "AND b.title ILIKE ?");
    assert_eq!(params, vec!["exact title"]);
}

#[test]
fn build_filter_conditions_title_empty_produces_nothing() {
    let filters = SearchFilters {
        title: Some(String::new()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.is_empty());
    assert!(params.is_empty());
}

#[test]
fn build_filter_conditions_authors_partial() {
    let filters = SearchFilters {
        authors: Some("John".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("books_authors ba"));
    assert!(
        sql.contains(
            "CONCAT_WS(' ', a.first_name, a.middle_name, a.last_name, a.nickname) ILIKE ?"
        )
    );
    assert_eq!(params, vec!["%John%"]);
}

#[test]
fn build_filter_conditions_authors_exact() {
    let filters = SearchFilters {
        authors: Some("=John Doe".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("CONCAT_WS"));
    assert_eq!(params, vec!["John Doe"]);
}

#[test]
fn build_filter_conditions_genres_partial() {
    let filters = SearchFilters {
        genres: Some("fantasy".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("book.genres"));
    assert!(sql.contains("ILIKE ?"));
    assert!(!sql.contains("json_each"));
    assert_eq!(params, vec!["%fantasy%"]);
}

#[test]
fn build_filter_conditions_genres_exact() {
    let filters = SearchFilters {
        genres: Some("=sci_fi".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("json_each"));
    assert!(sql.contains("json_extract_string"));
    assert_eq!(params, vec!["sci_fi"]);
}

#[test]
fn build_filter_conditions_date_partial() {
    let filters = SearchFilters {
        date: Some("2023".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("book.date"));
    assert!(sql.contains("ILIKE ?"));
    assert_eq!(params, vec!["%2023%"]);
}

#[test]
fn build_filter_conditions_date_exact() {
    let filters = SearchFilters {
        date: Some("=2023-05".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("book.date"));
    assert_eq!(params, vec!["2023-05"]);
}

#[test]
fn build_filter_conditions_sequence_partial() {
    let filters = SearchFilters {
        sequence: Some("Saga".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert_eq!(sql, "AND b.sequence ILIKE ?");
    assert_eq!(params, vec!["%Saga%"]);
}

#[test]
fn build_filter_conditions_sequence_exact() {
    let filters = SearchFilters {
        sequence: Some("=Exact Series".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert_eq!(sql, "AND b.sequence ILIKE ?");
    assert_eq!(params, vec!["Exact Series"]);
}

#[test]
fn build_filter_conditions_lang_single() {
    let filters = SearchFilters {
        lang: Some("en".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("book.lang"));
    assert!(sql.contains("ILIKE ?"));
    assert_eq!(params, vec!["%en%"]);
}

#[test]
fn build_filter_conditions_lang_exact() {
    let filters = SearchFilters {
        lang: Some("=en".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("book.lang"));
    assert_eq!(params, vec!["en"]);
}

#[test]
fn build_filter_conditions_lang_or_pattern() {
    let filters = SearchFilters {
        lang: Some("en|ru".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains(" OR "));
    assert_eq!(sql.matches("ILIKE ?").count(), 2);
    assert_eq!(params, vec!["en", "ru"]);
}

#[test]
fn build_filter_conditions_lang_or_with_empty() {
    let filters = SearchFilters {
        lang: Some("en|".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains(" OR "));
    assert!(sql.contains("= ''"));
    assert_eq!(params, vec!["en"]);
}

#[test]
fn build_filter_conditions_lang_triple_or() {
    let filters = SearchFilters {
        lang: Some("en|ru|de".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert_eq!(sql.matches(" OR ").count(), 2);
    assert_eq!(params, vec!["en", "ru", "de"]);
}

#[test]
fn build_filter_conditions_file_size_operator() {
    let filters = SearchFilters {
        file_size: Some(">300kb".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("book.file_size"));
    let expected_bytes = 300 * 1024;
    assert!(sql.contains(&format!("> {expected_bytes}")));
    assert!(params.is_empty());
}

#[test]
fn build_filter_conditions_file_size_text_partial() {
    let filters = SearchFilters {
        file_size: Some("123".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("CAST("));
    assert!(sql.contains("AS VARCHAR) ILIKE ?"));
    assert_eq!(params, vec!["%123%"]);
}

#[test]
fn build_filter_conditions_file_size_text_exact() {
    let filters = SearchFilters {
        file_size: Some("=123".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.contains("CAST("));
    assert_eq!(params, vec!["123"]);
}

#[test]
fn build_filter_conditions_file_size_empty() {
    let filters = SearchFilters {
        file_size: Some("   ".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.is_empty());
    assert!(params.is_empty());
}

#[test]
fn build_filter_conditions_lang_empty() {
    let filters = SearchFilters {
        lang: Some("   ".to_string()),
        ..SearchFilters::default()
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert!(sql.is_empty());
    assert!(params.is_empty());
}

#[test]
fn build_filter_conditions_multiple() {
    let filters = SearchFilters {
        title: Some("Book".to_string()),
        authors: Some("Author".to_string()),
        genres: Some("Fantasy".to_string()),
        date: Some("2023".to_string()),
        sequence: Some("Series".to_string()),
        lang: Some("en".to_string()),
        file_size: Some(">100kb".to_string()),
    };
    let (sql, params) = build_filter_conditions(&filters, "b");
    assert_eq!(sql.matches("AND ").count(), 7);
    assert_eq!(params.len(), 6);
}

#[test]
fn build_filter_conditions_table_alias_propagated() {
    let filters = SearchFilters {
        title: Some("test".to_string()),
        sequence: Some("seq".to_string()),
        ..SearchFilters::default()
    };
    let (sql, _) = build_filter_conditions(&filters, "my_alias");
    assert!(sql.contains("my_alias.title"));
    assert!(sql.contains("my_alias.sequence"));
}
