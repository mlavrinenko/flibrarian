use std::path::Path;

use flibrarian_core::common::get_db_path;

#[test]
fn test_get_db_path() {
    let path = Path::new("/some/library");
    assert_eq!(get_db_path(path), Path::new("/some/library/lib.duckdb"));
}

#[test]
fn test_get_db_path_relative() {
    let path = Path::new("relative/path");
    assert_eq!(get_db_path(path), Path::new("relative/path/lib.duckdb"));
}
