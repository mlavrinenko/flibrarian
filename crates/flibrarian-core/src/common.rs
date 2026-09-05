use anyhow::{Context, Result};
use duckdb::{AccessMode, Config, Connection};
use log::info;
use std::path::{Path, PathBuf};

#[must_use]
pub fn get_db_path(library_path: &Path) -> std::path::PathBuf {
    library_path.join("lib.duckdb")
}

pub fn db_config(mode: AccessMode) -> Result<Config> {
    let dir = std::env::var("FLIBRARIAN_DUCKDB_EXTENSION_DIR")
        .ok()
        .filter(|d| !d.is_empty());
    db_config_with_extensions(mode, dir.as_deref())
}

/// Sole owner of the extension-loading policy: `schema.sql` must not re-`SET`
/// these, or it would clobber the decision on every connection.
pub fn db_config_with_extensions(mode: AccessMode, extension_dir: Option<&str>) -> Result<Config> {
    let config = Config::default()
        .access_mode(mode)?
        .enable_autoload_extension(true)?;

    // Nix builds link the stock duckdb, which has no built-in fts, and stage the
    // official extension in the store. Autoinstall goes off there on purpose:
    // enable_autoload_extension turns it on, so a version mismatch would quietly
    // download a copy over the network instead of failing.
    match extension_dir {
        Some(dir) => Ok(config
            .with("extension_directory", dir)?
            .with("autoinstall_known_extensions", "false")?),
        None => Ok(config),
    }
}

#[must_use]
pub fn resolve_path(path: &str) -> PathBuf {
    let p = PathBuf::from(path);
    if p.is_absolute() {
        p
    } else {
        std::env::current_dir().map_or_else(|_| p.clone(), |cwd| cwd.join(&p))
    }
}

pub fn create_database_connection(db_path: &Path) -> Result<Connection> {
    info!("Connecting to database at {}", db_path.display());
    let conn = Connection::open_with_flags(db_path, db_config(AccessMode::ReadWrite)?)
        .context("Failed to open or create database file")?;

    conn.execute_batch(include_str!("schema.sql"))
        .context("Failed to initialize database schema")?;

    info!("Database schema initialized");
    Ok(conn)
}
