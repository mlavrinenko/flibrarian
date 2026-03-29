use super::*;

#[test]
fn extract_description_bytes_standard() {
    let input = b"<?xml?><FictionBook><description><title-info>data</title-info></description><body/></FictionBook>";
    let result = extract_description_bytes(input).unwrap();
    assert_eq!(
        std::str::from_utf8(result).unwrap(),
        "<description><title-info>data</title-info></description>"
    );
}

#[test]
fn extract_description_bytes_with_attributes() {
    let input = b"<description lang=\"en\"><title-info/></description>";
    let result = extract_description_bytes(input).unwrap();
    assert!(result.starts_with(b"<description"));
    assert!(result.ends_with(b"</description>"));
}

#[test]
fn extract_description_bytes_missing_open() {
    let input = b"<FictionBook></description></FictionBook>";
    assert!(extract_description_bytes(input).is_none());
}

#[test]
fn extract_description_bytes_missing_close() {
    let input = b"<description><title-info/>";
    assert!(extract_description_bytes(input).is_none());
}

#[test]
fn extract_description_bytes_empty_input() {
    assert!(extract_description_bytes(b"").is_none());
}

#[test]
fn build_template_produces_valid_structure() {
    let result = build_template(
        r#"<?xml version="1.0"?>"#,
        "<FictionBook>",
        "<description>test</description>",
    );
    assert!(result.starts_with("<?xml"));
    assert!(result.contains("<FictionBook>"));
    assert!(result.contains("<description>test</description>"));
    assert!(result.contains("<body></body>"));
    assert!(result.ends_with("</FictionBook>"));
}

#[test]
fn extract_xml_decl_from_short_input() {
    let input = b"<FictionBook/>";
    let (xml_decl, fb_tag) = extract_xml_declaration_and_fictionbook_tag(input);
    assert!(xml_decl.contains("UTF-8"));
    assert!(fb_tag.contains("FictionBook"));
}

#[test]
fn convert_fb2_author_whitespace_trimmed() {
    let author = PartialFb2Author {
        first_name: Some(vec![fb2::LocalizedText {
            lang: None,
            value: "  John  ".to_string(),
        }]),
        middle_name: None,
        last_name: Some(vec![fb2::LocalizedText {
            lang: None,
            value: "  Doe  ".to_string(),
        }]),
        nickname: None,
        id: Some("id".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result[0].first_name, Some("John".to_string()));
    assert_eq!(result[0].last_name, Some("Doe".to_string()));
}

#[test]
fn convert_fb2_author_whitespace_only_filtered() {
    let author = PartialFb2Author {
        first_name: Some(vec![fb2::LocalizedText {
            lang: None,
            value: "   ".to_string(),
        }]),
        middle_name: None,
        last_name: None,
        nickname: None,
        id: Some("id".to_string()),
    };
    let result = convert_fb2_author_to_struct(author);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].first_name, None);
}
