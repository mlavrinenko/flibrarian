use flibrarian_core::indexing::types::PartialFb2Author;
use flibrarian_core::indexing::xml::{
    convert_fb2_author_to_struct, create_template_with_description,
    extract_xml_declaration_and_fictionbook_tag,
};

fn localized(value: &str) -> fb2::LocalizedText {
    fb2::LocalizedText {
        lang: None,
        value: value.to_string(),
    }
}

#[test]
fn test_extract_xml_declaration_standard() {
    let input = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description></description>
</FictionBook>"#;

    let (xml_decl, fb_tag) = extract_xml_declaration_and_fictionbook_tag(input);
    assert!(xml_decl.contains("UTF-8"));
    assert!(fb_tag.starts_with("<FictionBook"));
}

#[test]
fn test_extract_xml_declaration_missing() {
    let input = b"<FictionBook><description></description></FictionBook>";
    let (xml_decl, _fb_tag) = extract_xml_declaration_and_fictionbook_tag(input);
    assert!(xml_decl.contains("UTF-8"));
}

#[test]
fn test_extract_fictionbook_tag_missing() {
    let input = br#"<?xml version="1.0" encoding="UTF-8"?><root></root>"#;
    let (_xml_decl, fb_tag) = extract_xml_declaration_and_fictionbook_tag(input);
    assert!(fb_tag.starts_with("<FictionBook"));
}

#[test]
fn test_create_template_with_description_found() {
    let content = r#"<?xml version="1.0"?><FictionBook><description><title-info><book-title>Test</book-title></title-info></description><body>text</body></FictionBook>"#;
    let result = create_template_with_description(
        content,
        r#"<?xml version="1.0" encoding="UTF-8"?>"#,
        r#"<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">"#,
    );
    assert!(result.contains("<description>"));
    assert!(result.contains("<book-title>Test</book-title>"));
    assert!(result.contains("</FictionBook>"));
    assert!(!result.contains("text</body>"));
}

#[test]
fn test_create_template_no_description() {
    let content = "<FictionBook><body>text</body></FictionBook>";
    let result =
        create_template_with_description(content, r#"<?xml version="1.0"?>"#, "<FictionBook>");
    assert!(result.contains("<book-title>Unknown</book-title>"));
}

#[test]
fn test_convert_author_full() {
    let author = PartialFb2Author {
        first_name: Some(vec![localized("John")]),
        middle_name: Some(vec![localized("M")]),
        last_name: Some(vec![localized("Doe")]),
        nickname: None,
        id: Some("author-1".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].first_name, Some("John".to_string()));
    assert_eq!(result[0].middle_name, Some("M".to_string()));
    assert_eq!(result[0].last_name, Some("Doe".to_string()));
    assert_eq!(result[0].id, "author-1");
}

#[test]
fn test_convert_author_empty_produces_anonymous() {
    let author = PartialFb2Author {
        first_name: None,
        middle_name: None,
        last_name: None,
        nickname: None,
        id: Some("id-1".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "id-1");
    assert_eq!(result[0].first_name, None);
}

#[test]
fn test_convert_author_no_id_generates_uuid() {
    let author = PartialFb2Author {
        first_name: Some(vec![localized("Jane")]),
        middle_name: None,
        last_name: None,
        nickname: None,
        id: None,
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 1);
    assert!(!result[0].id.is_empty());
    assert_eq!(result[0].first_name, Some("Jane".to_string()));
}

#[test]
fn test_convert_author_multiple_names_produces_multiple_authors() {
    let author = PartialFb2Author {
        first_name: Some(vec![localized("Alice"), localized("Bob")]),
        middle_name: None,
        last_name: Some(vec![localized("Smith"), localized("Jones")]),
        nickname: None,
        id: Some("shared-id".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].first_name, Some("Alice".to_string()));
    assert_eq!(result[0].last_name, Some("Smith".to_string()));
    assert_eq!(result[1].first_name, Some("Bob".to_string()));
    assert_eq!(result[1].last_name, Some("Jones".to_string()));
}

#[test]
fn test_convert_author_uneven_name_counts() {
    let author = PartialFb2Author {
        first_name: Some(vec![
            localized("Alice"),
            localized("Bob"),
            localized("Charlie"),
        ]),
        middle_name: None,
        last_name: Some(vec![localized("Smith")]),
        nickname: None,
        id: Some("id".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].last_name, Some("Smith".to_string()));
    assert_eq!(result[1].last_name, None);
    assert_eq!(result[2].last_name, None);
}

#[test]
fn test_convert_author_empty_strings_filtered() {
    let author = PartialFb2Author {
        first_name: Some(vec![localized("")]),
        middle_name: Some(vec![localized("")]),
        last_name: Some(vec![localized("")]),
        nickname: Some(vec![localized("")]),
        id: None,
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].first_name, None);
    assert_eq!(result[0].last_name, None);
}

#[test]
fn test_convert_author_nickname_only() {
    let author = PartialFb2Author {
        first_name: None,
        middle_name: None,
        last_name: None,
        nickname: Some(vec![localized("CoolNick")]),
        id: Some("nick-id".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].nickname, Some("CoolNick".to_string()));
    assert_eq!(result[0].first_name, None);
}
