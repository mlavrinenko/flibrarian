use super::*;

#[test]
fn parse_filter_value_exact_match() {
    let (exact, value) = parse_filter_value("=specific");
    assert!(exact);
    assert_eq!(value, "specific");
}

#[test]
fn parse_filter_value_partial_match() {
    let (exact, value) = parse_filter_value("partial");
    assert!(!exact);
    assert_eq!(value, "partial");
}

#[test]
fn parse_filter_value_with_whitespace() {
    let (exact, value) = parse_filter_value("  =exact  ");
    assert!(exact);
    assert_eq!(value, "exact");
}

#[test]
fn parse_filter_value_empty_string() {
    let (exact, value) = parse_filter_value("");
    assert!(!exact);
    assert_eq!(value, "");
}

#[test]
fn parse_filter_value_equals_only() {
    let (exact, value) = parse_filter_value("=");
    assert!(exact);
    assert_eq!(value, "");
}

#[test]
fn parse_file_size_gt_kb() {
    let (op, bytes) = parse_file_size_filter(">300kb").unwrap();
    assert_eq!(op, ">");
    assert_eq!(bytes, 300 * 1024);
}

#[test]
fn parse_file_size_lt_mb() {
    let (op, bytes) = parse_file_size_filter("<1Mb").unwrap();
    assert_eq!(op, "<");
    assert_eq!(bytes, 1024 * 1024);
}

#[test]
fn parse_file_size_gte_with_spaces() {
    let (op, bytes) = parse_file_size_filter(">= 500 kb").unwrap();
    assert_eq!(op, ">=");
    assert_eq!(bytes, 500 * 1024);
}

#[test]
fn parse_file_size_lte() {
    let (op, bytes) = parse_file_size_filter("<= 2 mb").unwrap();
    assert_eq!(op, "<=");
    assert_eq!(bytes, 2 * 1024 * 1024);
}

#[test]
fn parse_file_size_bare_bytes() {
    let (op, bytes) = parse_file_size_filter("<1000").unwrap();
    assert_eq!(op, "<");
    assert_eq!(bytes, 1000);
}

#[test]
fn parse_file_size_explicit_b_unit() {
    let (op, bytes) = parse_file_size_filter(">500b").unwrap();
    assert_eq!(op, ">");
    assert_eq!(bytes, 500);
}

#[test]
fn parse_file_size_k_shorthand() {
    let (op, bytes) = parse_file_size_filter(">10k").unwrap();
    assert_eq!(op, ">");
    assert_eq!(bytes, 10 * 1024);
}

#[test]
fn parse_file_size_m_shorthand() {
    let (op, bytes) = parse_file_size_filter("<5m").unwrap();
    assert_eq!(op, "<");
    assert_eq!(bytes, 5 * 1024 * 1024);
}

#[test]
fn parse_file_size_gb() {
    let (op, bytes) = parse_file_size_filter(">1g").unwrap();
    assert_eq!(op, ">");
    assert_eq!(bytes, 1024 * 1024 * 1024);
}

#[test]
fn parse_file_size_no_operator() {
    assert!(parse_file_size_filter("300kb").is_none());
}

#[test]
fn parse_file_size_invalid_unit() {
    assert!(parse_file_size_filter(">300xx").is_none());
}

#[test]
fn parse_file_size_empty() {
    assert!(parse_file_size_filter("").is_none());
}
