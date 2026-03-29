use flibrarian_core::indexing::{deserialize_fb2, parse_book_from_bytes};

#[test]
fn deserialize_fb2_fallback_when_description_tag_unclosed() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>Broken Description</book-title>
</title-info>
<body></body>
</FictionBook>"#;

    let result = deserialize_fb2(xml);
    assert!(
        result.is_ok(),
        "Expected fallback parsing to succeed: {result:?}"
    );
    let fb = result.unwrap();
    assert!(!fb.description.title_info.is_empty());
}

#[test]
fn deserialize_fb2_no_description_tag_at_all() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<body><section><p>No description here</p></section></body>
</FictionBook>"#;

    let result = deserialize_fb2(xml);
    assert!(
        result.is_ok(),
        "Expected fallback with dummy description: {result:?}"
    );
}

#[test]
fn parse_book_from_bytes_no_title_info_fails() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description></description>
<body></body>
</FictionBook>"#;

    let result = parse_book_from_bytes(1, xml);
    assert!(result.is_err());
}

#[test]
fn deserialize_fb2_missing_xml_declaration() {
    let xml = br#"<FictionBook xmlns="http://www.gribuser.ru/xml/fictionbook/2.0">
<description>
<title-info>
<book-title>No XML Decl</book-title>
</title-info>
</description>
<body></body>
</FictionBook>"#;

    let result = deserialize_fb2(xml);
    assert!(result.is_ok());
}

#[test]
fn deserialize_fb2_missing_fictionbook_tag_uses_default() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<description>
<title-info>
<book-title>No FictionBook Tag</book-title>
</title-info>
</description>
<body></body>"#;

    let result = deserialize_fb2(xml);
    assert!(result.is_ok());
}

#[test]
fn parse_book_with_non_utf8_encoding() {
    let header = b"<?xml version=\"1.0\" encoding=\"windows-1251\"?>\n";
    let body = b"<FictionBook xmlns=\"http://www.gribuser.ru/xml/fictionbook/2.0\">\n\
        <description>\n\
        <title-info>\n\
        <book-title>";

    let title_bytes: &[u8] = &[0xD2, 0xE5, 0xF1, 0xF2]; // "Тест" in windows-1251
    let rest = b"</book-title>\n\
        </title-info>\n\
        </description>\n\
        <body></body>\n\
        </FictionBook>";

    let mut raw = Vec::new();
    raw.extend_from_slice(header);
    raw.extend_from_slice(body);
    raw.extend_from_slice(title_bytes);
    raw.extend_from_slice(rest);

    let result = parse_book_from_bytes(42, &raw);
    assert!(
        result.is_ok(),
        "Expected encoding conversion to handle win-1251: {result:?}"
    );
}
