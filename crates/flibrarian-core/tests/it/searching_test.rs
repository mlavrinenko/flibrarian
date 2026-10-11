use flibrarian_core::searching::{value_ref_to_vec_author_struct, value_ref_to_vec_string};

use duckdb::types::Value;

fn make_value_ref_text(text: &str) -> Value {
    Value::Text(text.to_string())
}

#[test]
fn test_value_ref_to_vec_string_valid_json() {
    let val = make_value_ref_text(r#"["fantasy","adventure"]"#);
    let value_ref = duckdb::types::ValueRef::from(&val);
    let result = value_ref_to_vec_string(value_ref).unwrap();
    assert_eq!(result, vec!["fantasy", "adventure"]);
}

#[test]
fn test_value_ref_to_vec_string_empty_array() {
    let val = make_value_ref_text("[]");
    let value_ref = duckdb::types::ValueRef::from(&val);
    let result = value_ref_to_vec_string(value_ref).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_value_ref_to_vec_string_invalid_json() {
    let val = make_value_ref_text("not json");
    let value_ref = duckdb::types::ValueRef::from(&val);
    assert!(value_ref_to_vec_string(value_ref).is_err());
}

#[test]
fn test_value_ref_to_vec_string_wrong_type() {
    let val = Value::Int(42);
    let value_ref = duckdb::types::ValueRef::from(&val);
    assert!(value_ref_to_vec_string(value_ref).is_err());
}

#[test]
fn test_value_ref_to_vec_author_struct_valid() {
    let json =
        r#"[{"id":"1","first_name":"John","middle_name":null,"last_name":"Doe","nickname":null}]"#;
    let val = make_value_ref_text(json);
    let value_ref = duckdb::types::ValueRef::from(&val);
    let result = value_ref_to_vec_author_struct(value_ref).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].first_name, Some("John".to_string()));
    assert_eq!(result[0].last_name, Some("Doe".to_string()));
}

#[test]
fn test_value_ref_to_vec_author_struct_null_returns_empty() {
    let val = Value::Null;
    let value_ref = duckdb::types::ValueRef::from(&val);
    let result = value_ref_to_vec_author_struct(value_ref).unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_value_ref_to_vec_author_struct_wrong_type() {
    let val = Value::Int(42);
    let value_ref = duckdb::types::ValueRef::from(&val);
    assert!(value_ref_to_vec_author_struct(value_ref).is_err());
}

#[test]
fn test_value_ref_to_vec_author_struct_invalid_json() {
    let val = make_value_ref_text("not json");
    let value_ref = duckdb::types::ValueRef::from(&val);
    assert!(value_ref_to_vec_author_struct(value_ref).is_err());
}
