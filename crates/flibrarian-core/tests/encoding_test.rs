use flibrarian_core::encoding::{Utf8Reader, to_utf8_reader};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

#[test]
fn test_utf8_detection_with_fb2_file() {
    // Test with the provided FB2 file
    let file = File::open("tests/res/utf16_encoded.fb2").expect("Failed to open test file");
    let buffered_reader = BufReader::new(file);

    match to_utf8_reader(buffered_reader) {
        Ok(Utf8Reader::Utf8(_)) => {
            panic!("UTF-16 file should not be detected as UTF-8");
        }
        Ok(Utf8Reader::NonUtf8(_)) => {}
        Err(e) => {
            panic!("Failed to create UTF-8 reader: {e}");
        }
    }
}

#[test]
fn test_actual_content_reading() {
    let file = File::open("tests/res/utf16_encoded.fb2").expect("Failed to open test file");
    let buffered_reader = BufReader::new(file);

    let mut reader = to_utf8_reader(buffered_reader).expect("Failed to create UTF-8 reader");
    let mut content = String::new();
    reader
        .read_to_string(&mut content)
        .expect("Failed to read content");

    let bytes = content.as_bytes();
    let mut has_null_interleaved = false;
    for i in (1..std::cmp::min(20, bytes.len())).step_by(2) {
        if bytes[i] == 0 {
            has_null_interleaved = true;
            break;
        }
    }

    assert!(
        !has_null_interleaved,
        "Content still appears to be improperly decoded UTF-16"
    );

    assert!(
        content.starts_with("<?xml"),
        "Content should start with XML declaration"
    );
}

#[test]
fn test_encoding_detection() {
    // Test UTF-8 file
    let utf8_content = "Hello, world! Привет, мир! 🦀".as_bytes();
    let cursor = std::io::Cursor::new(utf8_content.to_vec());
    let buffered_reader = BufReader::new(cursor);

    match to_utf8_reader(buffered_reader) {
        Ok(Utf8Reader::Utf8(_)) => {
            println!("UTF-8 content correctly detected as UTF-8");
        }
        Ok(Utf8Reader::NonUtf8(_)) => {
            panic!("UTF-8 content incorrectly detected as non-UTF-8");
        }
        Err(e) => {
            panic!("Failed to create UTF-8 reader for UTF-8 content: {e}");
        }
    }

    // Test Windows-1251 encoded content (approximation)
    // This is a byte sequence that represents "Привет" in Windows-1251
    let win1251_content = [0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2]; // "Привет" in Windows-1251
    let cursor = std::io::Cursor::new(win1251_content.to_vec());
    let buffered_reader = BufReader::new(cursor);

    match to_utf8_reader(buffered_reader) {
        Ok(Utf8Reader::Utf8(_)) => {
            println!(
                "Non-UTF-8 content detected as UTF-8 (might be correct if it happens to be valid UTF-8)"
            );
        }
        Ok(Utf8Reader::NonUtf8(_)) => {
            println!("Non-UTF-8 content correctly detected as non-UTF-8");
        }
        Err(e) => {
            panic!("Failed to create UTF-8 reader for non-UTF-8 content: {e}");
        }
    }
}

#[test]
fn test_manual_utf16_detection() {
    let file = File::open("tests/res/utf16_encoded.fb2").expect("Failed to open test file");
    let mut buffered_reader = BufReader::new(file);
    let mut sample = [0u8; 10];
    buffered_reader
        .read_exact(&mut sample)
        .expect("Failed to read sample");

    println!("First 10 bytes: {sample:02X?}");

    // Check for UTF-16 BOM
    if sample.starts_with(&[0xFF, 0xFE]) {
        println!("File has UTF-16 LE BOM");
    } else if sample.starts_with(&[0xFE, 0xFF]) {
        println!("File has UTF-16 BE BOM");
    } else {
        println!("File has no recognizable BOM");
    }
}

#[test]
fn test_utf16_le_decoding() {
    // Test explicit UTF-16 LE decoding with BOM
    let utf16_le_content = b"\xFF\xFEH\x00e\x00l\x00l\x00o\x00"; // "Hello" in UTF-16 LE with BOM
    let cursor = std::io::Cursor::new(utf16_le_content.to_vec());
    let buffered_reader = BufReader::new(cursor);

    let mut reader = to_utf8_reader(buffered_reader).expect("Failed to create UTF-8 reader");
    let mut content = String::new();

    match reader.read_to_string(&mut content) {
        Ok(_) => {
            assert_eq!(content, "Hello");
            println!("UTF-16 LE with BOM correctly decoded to: {content}");
        }
        Err(e) => {
            panic!("Failed to read UTF-16 LE content: {e}");
        }
    }
}

#[test]
fn test_utf8_bom_stripped() {
    let content = b"\xEF\xBB\xBFHello UTF-8 BOM";
    let cursor = std::io::Cursor::new(content.to_vec());
    let reader = BufReader::new(cursor);

    let mut utf8_reader = to_utf8_reader(reader).unwrap();
    let mut result = String::new();
    utf8_reader.read_to_string(&mut result).unwrap();
    assert_eq!(result, "Hello UTF-8 BOM");
    assert!(matches!(utf8_reader, Utf8Reader::Utf8(_)));
}

#[test]
fn test_utf16_be_bom_decoded() {
    let content = b"\xFE\xFF\x00H\x00e\x00l\x00l\x00o";
    let cursor = std::io::Cursor::new(content.to_vec());
    let reader = BufReader::new(cursor);

    let mut utf8_reader = to_utf8_reader(reader).unwrap();
    let mut result = String::new();
    utf8_reader.read_to_string(&mut result).unwrap();
    assert_eq!(result, "Hello");
}

#[test]
fn test_bufread_fill_buf_and_consume_utf8() {
    let content = b"line1\nline2\nline3";
    let cursor = std::io::Cursor::new(content.to_vec());
    let reader = BufReader::new(cursor);

    let mut utf8_reader = to_utf8_reader(reader).unwrap();
    let mut line = String::new();
    utf8_reader.read_line(&mut line).unwrap();
    assert_eq!(line, "line1\n");

    line.clear();
    utf8_reader.read_line(&mut line).unwrap();
    assert_eq!(line, "line2\n");
}

#[test]
fn test_empty_input() {
    let cursor = std::io::Cursor::new(Vec::<u8>::new());
    let reader = BufReader::new(cursor);

    let mut utf8_reader = to_utf8_reader(reader).unwrap();
    let mut result = String::new();
    utf8_reader.read_to_string(&mut result).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_plain_ascii_detected_as_utf8() {
    let content = b"Just plain ASCII text";
    let cursor = std::io::Cursor::new(content.to_vec());
    let reader = BufReader::new(cursor);

    let utf8_reader = to_utf8_reader(reader).unwrap();
    assert!(matches!(utf8_reader, Utf8Reader::Utf8(_)));
}

#[test]
fn test_bufread_fill_buf_and_consume_non_utf8() {
    let utf16_le = b"\xFF\xFEH\x00e\x00l\x00l\x00o\x00\n\x00W\x00o\x00r\x00l\x00d\x00";
    let cursor = std::io::Cursor::new(utf16_le.to_vec());
    let reader = BufReader::new(cursor);

    let mut utf8_reader = to_utf8_reader(reader).unwrap();
    assert!(matches!(utf8_reader, Utf8Reader::NonUtf8(_)));

    let mut line = String::new();
    utf8_reader.read_line(&mut line).unwrap();
    assert_eq!(line, "Hello\n");

    line.clear();
    utf8_reader.read_line(&mut line).unwrap();
    assert_eq!(line, "World");
}

#[test]
fn test_chardetng_detection_win1251() {
    let win1251: Vec<u8> = vec![
        0xCF, 0xF0, 0xE8, 0xE2, 0xE5, 0xF2, 0x2C, 0x20, 0xEC, 0xE8, 0xF0, 0x21, 0x20, 0xDD, 0xF2,
        0xEE, 0x20, 0xF2, 0xE5, 0xF1, 0xF2, 0x20, 0xED, 0xE0, 0x20, 0xEA, 0xEE, 0xE4, 0xE8, 0xF0,
        0xEE, 0xE2, 0xEA, 0xF3, 0x20, 0xC2, 0xE8, 0xED, 0xE4, 0xEE, 0xF3, 0xE7, 0x2D, 0x31, 0x32,
        0x35, 0x31,
    ];
    let cursor = std::io::Cursor::new(win1251);
    let reader = BufReader::new(cursor);

    let mut utf8_reader = to_utf8_reader(reader).unwrap();
    assert!(matches!(utf8_reader, Utf8Reader::NonUtf8(_)));

    let mut result = String::new();
    utf8_reader.read_to_string(&mut result).unwrap();
    assert!(result.contains("Привет"));
}
