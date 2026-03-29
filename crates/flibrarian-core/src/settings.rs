use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiSettings {
    pub dock_height: Option<u32>,
    pub column_widths: Option<HashMap<String, u32>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub library_path: Option<String>,
    pub default_save_folder: Option<String>,
    pub ui: Option<UiSettings>,
}

pub fn settings_path() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|dir| dir.join("flibrarian").join("settings.toml"))
        .context("Could not determine config directory")
}

pub fn load_settings() -> Result<Settings> {
    load_settings_from(&settings_path()?)
}

pub fn save_settings(settings: &Settings) -> Result<()> {
    save_settings_to(&settings_path()?, settings)
}

pub fn load_settings_from(path: &Path) -> Result<Settings> {
    if !path.exists() {
        return Ok(Settings::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read settings from {}", path.display()))?;

    toml::from_str(&content)
        .with_context(|| format!("Failed to parse settings from {}", path.display()))
}

pub fn save_settings_to(path: &Path, settings: &Settings) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create config directory {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(settings).context("Failed to serialize settings")?;

    fs::write(path, content)
        .with_context(|| format!("Failed to write settings to {}", path.display()))
}
