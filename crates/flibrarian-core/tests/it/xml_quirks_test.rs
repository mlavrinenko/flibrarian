use flibrarian_core::indexing::parse_book_from_bytes;
use std::fs;

fn read_test_fb2(filename: &str) -> Vec<u8> {
    fs::read(format!("tests/res/{filename}")).unwrap()
}

#[test]
fn test_process_fb2_file_success() {
    let book = parse_book_from_bytes(313_077, &read_test_fb2("broken_structure.fb2")).unwrap();

    assert_eq!(book.id, 313_077);
    assert_eq!(book.title, "Тайны звёздного неба");
    assert_eq!(book.date, "15.03.2020");
    assert_eq!(book.sequence, "");
    assert_eq!(book.genres, vec!["science"]);

    assert_eq!(book.authors.len(), 1);
    let author = &book.authors[0];
    assert_eq!(author.first_name, Some("Алексей".to_string()));
    assert_eq!(author.last_name, Some("Волков".to_string()));
    assert_eq!(author.nickname, None);
}

#[test]
fn test_broken_xml_structure() {
    let book = parse_book_from_bytes(313_078, &read_test_fb2("broken_structure.fb2")).unwrap();

    assert_eq!(book.id, 313_078);
    assert_eq!(book.title, "Тайны звёздного неба");
    assert_eq!(book.date, "15.03.2020");
}

#[test]
fn test_utf16_encoding() {
    let book = parse_book_from_bytes(313_079, &read_test_fb2("utf16_encoded.fb2")).unwrap();

    assert_eq!(book.id, 313_079);
    assert_eq!(book.title, "Тайны звёздного неба");
    assert_eq!(book.date, "15.03.2020");
}

#[test]
fn test_multiple_title_info_elements() {
    let book = parse_book_from_bytes(313_080, &read_test_fb2("multiple_elements.fb2")).unwrap();

    assert_eq!(book.id, 313_080);
    let author = &book.authors[0];
    assert_eq!(author.first_name, Some("Дмитрий".to_string()));
    assert_eq!(author.last_name, Some("Орлов".to_string()));
    assert_eq!(book.title, "Тайны звёздного неба Путеводитель наблюдателя");
    assert_eq!(book.date, "2020");
}

#[test]
fn test_multiple_book_titles() {
    let book = parse_book_from_bytes(313_081, &read_test_fb2("multiple_elements.fb2")).unwrap();

    assert_eq!(book.id, 313_081);
    assert_eq!(book.title, "Тайны звёздного неба Путеводитель наблюдателя");
    assert_eq!(book.date, "2020");
}

#[test]
fn test_broken_author_structure() {
    let book =
        parse_book_from_bytes(313_082, &read_test_fb2("broken_author_structure.fb2")).unwrap();

    assert_eq!(book.id, 313_082);
    assert_eq!(book.title, "Тайны звёздного неба");
    assert_eq!(book.date, "15.03.2020");
    assert_eq!(book.authors.len(), 3);
}

#[test]
fn test_date_priority_in_different_blocks() {
    let book = parse_book_from_bytes(313_083, &read_test_fb2("date_in_publish_info.fb2")).unwrap();

    assert_eq!(book.id, 313_083);
    assert_eq!(book.title, "Тайны звёздного неба");
    assert_eq!(book.date, "2020");
}

#[test]
fn test_no_date_available() {
    let book = parse_book_from_bytes(313_084, &read_test_fb2("no_date.fb2")).unwrap();

    assert_eq!(book.id, 313_084);
    assert_eq!(book.title, "Тайны звёздного неба");
}

#[test]
fn test_empty_fb2_file() {
    let book = parse_book_from_bytes(313_085, &read_test_fb2("empty_fb2.fb2")).unwrap();
    assert_eq!(book.id, 313_085);
    assert_eq!(book.title, "Unknown");
}

#[test]
fn test_overlapped_elements_structure() {
    let book = parse_book_from_bytes(313_086, &read_test_fb2("overlapped_elements.fb2")).unwrap();

    assert_eq!(book.id, 313_086);
    assert_eq!(book.title, "Тайны звёздного неба Путеводитель наблюдателя");
    assert_eq!(book.date, "15.03.2020");
}

#[test]
fn test_missing_book_title_in_secondary_title_info() {
    let book = parse_book_from_bytes(
        313_087,
        &read_test_fb2("missing_book_title_in_secondary_title_info.fb2"),
    )
    .unwrap();

    assert_eq!(book.title, "Тайны звёздного неба Путеводитель наблюдателя");
}
