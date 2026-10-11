use flibrarian_core::preflight::{WriteAccess, check_write_access, ensure_writable};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::TempDir;

fn set_mode(path: &Path, mode: u32) {
    fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("set permissions");
}

/// Root ignores the permission bits these tests rely on, so a test that would
/// otherwise assert "not writable" must be skipped instead of passing for the
/// wrong reason.
fn writes_ignore_permissions(read_only_file: &Path) -> bool {
    fs::OpenOptions::new()
        .write(true)
        .open(read_only_file)
        .is_ok()
}

#[test]
fn empty_writable_directory_is_writable() {
    let dir = TempDir::new().expect("tempdir");

    assert_eq!(check_write_access(dir.path()), WriteAccess::Writable);
    assert!(ensure_writable(dir.path()).is_ok());
}

#[test]
fn missing_library_is_reported_missing() {
    let dir = TempDir::new().expect("tempdir");
    let absent = dir.path().join("no-such-library");

    assert_eq!(check_write_access(&absent), WriteAccess::LibraryMissing);
    assert!(ensure_writable(&absent).is_err());
}

#[test]
fn writable_database_is_writable() {
    let dir = TempDir::new().expect("tempdir");
    fs::write(dir.path().join("lib.duckdb"), b"").expect("create db");

    assert_eq!(check_write_access(dir.path()), WriteAccess::Writable);
}

#[test]
fn read_only_database_is_reported_read_only() {
    let dir = TempDir::new().expect("tempdir");
    let db = dir.path().join("lib.duckdb");
    fs::write(&db, b"").expect("create db");
    set_mode(&db, 0o444);

    if writes_ignore_permissions(&db) {
        eprintln!("skipping: this process bypasses file permissions");
        return;
    }

    assert_eq!(
        check_write_access(dir.path()),
        WriteAccess::DatabaseReadOnly
    );

    let err = ensure_writable(dir.path()).expect_err("read-only database must fail preflight");
    let message = format!("{err:#}");
    assert!(message.contains(&dir.path().display().to_string()));
    assert!(message.contains("database_read_only"));
}

#[test]
fn read_only_directory_is_reported_read_only() {
    let dir = TempDir::new().expect("tempdir");
    let probe = dir.path().join("probe");
    fs::write(&probe, b"").expect("create probe");
    set_mode(&probe, 0o444);
    let bypasses = writes_ignore_permissions(&probe);
    fs::remove_file(&probe).expect("remove probe");

    if bypasses {
        eprintln!("skipping: this process bypasses file permissions");
        return;
    }

    set_mode(dir.path(), 0o555);
    let access = check_write_access(dir.path());
    set_mode(dir.path(), 0o755);

    assert_eq!(access, WriteAccess::DirectoryReadOnly);
}

#[test]
fn probe_file_is_not_left_behind() {
    let dir = TempDir::new().expect("tempdir");

    assert!(check_write_access(dir.path()).is_writable());

    let leftovers: Vec<_> = fs::read_dir(dir.path())
        .expect("read dir")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(leftovers.is_empty(), "left behind: {leftovers:?}");
}
