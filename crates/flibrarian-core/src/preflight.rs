use anyhow::{Result, bail};
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::Path;

use crate::common::get_db_path;

/// Whether a library can be indexed, as opposed to merely searched.
///
/// Searching opens the database read-only, so a library can serve queries
/// perfectly while every write fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteAccess {
    Writable,
    LibraryMissing,
    DatabaseReadOnly,
    DirectoryReadOnly,
}

impl WriteAccess {
    #[must_use]
    pub const fn is_writable(self) -> bool {
        matches!(self, Self::Writable)
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Writable => "writable",
            Self::LibraryMissing => "library_missing",
            Self::DatabaseReadOnly => "database_read_only",
            Self::DirectoryReadOnly => "directory_read_only",
        }
    }

    #[must_use]
    pub const fn explain(self) -> &'static str {
        match self {
            Self::Writable => "library is writable",
            Self::LibraryMissing => "library directory does not exist",
            Self::DatabaseReadOnly => "database file is not writable by this process",
            Self::DirectoryReadOnly => "library directory is not writable by this process",
        }
    }
}

/// Probes the access indexing needs, changing nothing.
///
/// An existing database is opened for writing but never truncated; a missing
/// one is stood in for by a probe file in the directory that must hold it.
#[must_use]
pub fn check_write_access(library_path: &Path) -> WriteAccess {
    if !library_path.is_dir() {
        return WriteAccess::LibraryMissing;
    }

    let db_path = get_db_path(library_path);
    if db_path.exists() {
        return if OpenOptions::new().write(true).open(&db_path).is_ok() {
            WriteAccess::Writable
        } else {
            WriteAccess::DatabaseReadOnly
        };
    }

    probe_directory(library_path)
}

fn probe_directory(library_path: &Path) -> WriteAccess {
    let probe = library_path.join(format!(".flibrarian-write-probe-{}", std::process::id()));

    match OpenOptions::new().write(true).create_new(true).open(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            WriteAccess::Writable
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => WriteAccess::Writable,
        Err(_) => WriteAccess::DirectoryReadOnly,
    }
}

/// Fails before any long-running write is attempted.
///
/// Names the path and the reason, so a permission problem reads as one instead
/// of as a database error thousands of log lines later.
pub fn ensure_writable(library_path: &Path) -> Result<()> {
    let access = check_write_access(library_path);
    if access.is_writable() {
        return Ok(());
    }

    bail!(
        "Cannot index {}: {} ({})",
        library_path.display(),
        access.explain(),
        access.as_str()
    )
}
