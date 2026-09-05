use anyhow::Result;
use duckdb::{AccessMode, Connection};
use flibrarian_core::common::db_config_with_extensions;

fn setting(conn: &Connection, name: &str) -> Result<String> {
    Ok(conn.query_row(
        "SELECT value FROM duckdb_settings() WHERE name = ?",
        [name],
        |row| row.get(0),
    )?)
}

fn open_with_schema(extension_dir: Option<&str>) -> Result<Connection> {
    let conn = Connection::open_in_memory_with_flags(db_config_with_extensions(
        AccessMode::ReadWrite,
        extension_dir,
    )?)?;
    conn.execute_batch(include_str!("../src/schema.sql"))?;
    Ok(conn)
}

#[test]
fn packaged_extension_dir_keeps_autoinstall_off_after_schema() -> Result<()> {
    let conn = open_with_schema(Some("/nonexistent/duckdb-extensions"))?;

    assert_eq!(setting(&conn, "autoinstall_known_extensions")?, "false");
    assert_eq!(setting(&conn, "autoload_known_extensions")?, "true");
    assert_eq!(
        setting(&conn, "extension_directory")?,
        "/nonexistent/duckdb-extensions"
    );
    Ok(())
}

#[test]
fn without_packaged_extensions_autoinstall_stays_on() -> Result<()> {
    let conn = open_with_schema(None)?;

    assert_eq!(setting(&conn, "autoinstall_known_extensions")?, "true");
    assert_eq!(setting(&conn, "autoload_known_extensions")?, "true");
    Ok(())
}
