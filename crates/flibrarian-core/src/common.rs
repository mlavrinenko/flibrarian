use anyhow::{Context, Result};
use duckdb::{AccessMode, Config, Connection};
use log::info;
use std::path::{Path, PathBuf};

#[must_use]
pub fn get_db_path(library_path: &Path) -> std::path::PathBuf {
    library_path.join("lib.duckdb")
}

pub fn db_config(mode: AccessMode) -> Result<Config> {
    Ok(Config::default()
        .access_mode(mode)?
        .enable_autoload_extension(true)?)
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
