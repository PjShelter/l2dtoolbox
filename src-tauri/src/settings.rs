use anyhow::{Context, Result};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

use crate::types::AppSettings;

fn settings_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .context("failed to locate app data directory")?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join("settings.json"))
}

pub fn load_settings(app: &AppHandle) -> Result<AppSettings> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let text = fs::read_to_string(path)?;
    let settings = serde_json::from_str::<AppSettings>(&text)?;
    Ok(settings)
}

pub fn save_settings(app: &AppHandle, mut settings: AppSettings) -> Result<AppSettings> {
    dedupe_recent_paths(&mut settings);
    let path = settings_path(app)?;
    let text = serde_json::to_string_pretty(&settings)?;
    fs::write(path, text)?;
    Ok(settings)
}

fn dedupe_recent_paths(settings: &mut AppSettings) {
    let mut deduped = Vec::new();
    for path in settings.recent_paths.iter() {
        if !path.trim().is_empty() && !deduped.iter().any(|item| item == path) {
            deduped.push(path.clone());
        }
    }
    deduped.truncate(12);
    settings.recent_paths = deduped;
}
