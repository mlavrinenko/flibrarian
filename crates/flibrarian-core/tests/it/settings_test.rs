use flibrarian_core::settings::{Settings, load_settings_from, save_settings_to};

#[test]
fn load_returns_default_when_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.toml");

    let settings = load_settings_from(&path).unwrap();

    assert_eq!(settings, Settings::default());
    assert!(settings.library_path.is_none());
    assert!(settings.default_save_folder.is_none());
}

#[test]
fn save_and_load_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.toml");

    let original = Settings {
        library_path: Some("/home/user/books".to_string()),
        default_save_folder: Some("/home/user/extracted".to_string()),
        ..Settings::default()
    };

    save_settings_to(&path, &original).unwrap();
    let loaded = load_settings_from(&path).unwrap();

    assert_eq!(loaded, original);
}

#[test]
fn save_creates_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("deep").join("settings.toml");

    let settings = Settings {
        library_path: Some("/tmp/lib".to_string()),
        default_save_folder: None,
        ..Settings::default()
    };

    save_settings_to(&path, &settings).unwrap();

    assert!(path.exists());
    let loaded = load_settings_from(&path).unwrap();
    assert_eq!(loaded, settings);
}

#[test]
fn save_overwrites_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.toml");

    let first = Settings {
        library_path: Some("/first".to_string()),
        default_save_folder: Some("/first/out".to_string()),
        ..Settings::default()
    };
    save_settings_to(&path, &first).unwrap();

    let second = Settings {
        library_path: Some("/second".to_string()),
        default_save_folder: None,
        ..Settings::default()
    };
    save_settings_to(&path, &second).unwrap();

    let loaded = load_settings_from(&path).unwrap();
    assert_eq!(loaded, second);
}

#[test]
fn load_partial_settings() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.toml");

    std::fs::write(&path, "library_path = \"/only/this\"\n").unwrap();

    let loaded = load_settings_from(&path).unwrap();
    assert_eq!(loaded.library_path, Some("/only/this".to_string()));
    assert!(loaded.default_save_folder.is_none());
}
