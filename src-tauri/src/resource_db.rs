use anyhow::{Context, Result};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

use crate::types::{ResourceDatabase, ResourceEntry};

fn database_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .context("failed to locate app data directory")?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join("resource_database.json"))
}

pub fn load_resource_database(app: &AppHandle) -> Result<ResourceDatabase> {
    let path = database_path(app)?;
    if !path.exists() {
        return Ok(ResourceDatabase::default());
    }

    let text = fs::read_to_string(path)?;
    let mut database = serde_json::from_str::<ResourceDatabase>(&text)?;
    database.entries.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| left.name.cmp(&right.name))
    });
    Ok(database)
}

pub fn upsert_resource_entry(
    app: &AppHandle,
    mut entry: ResourceEntry,
) -> Result<ResourceDatabase> {
    if entry.id.trim().is_empty() {
        anyhow::bail!("Resource id is required.");
    }
    if entry.name.trim().is_empty() {
        entry.name = entry.id.clone();
    }
    if entry.kind.trim().is_empty() {
        entry.kind = "live2d".to_string();
    }
    if entry.source.trim().is_empty() {
        entry.source = "manual".to_string();
    }

    let mut database = load_resource_database(app)?;
    let now = timestamp();
    if let Some(existing) = database.entries.iter_mut().find(|item| item.id == entry.id) {
        if entry.created_at.is_none() {
            entry.created_at = existing.created_at.clone().or_else(|| Some(now.clone()));
        }
        entry.updated_at = Some(now);
        *existing = entry;
    } else {
        entry.created_at = entry.created_at.or_else(|| Some(now.clone()));
        entry.updated_at = Some(now);
        database.entries.push(entry);
    }
    save_resource_database(app, &database)?;
    load_resource_database(app)
}

pub fn remove_resource_entry(app: &AppHandle, id: &str) -> Result<ResourceDatabase> {
    let mut database = load_resource_database(app)?;
    database.entries.retain(|entry| entry.id != id);
    save_resource_database(app, &database)?;
    load_resource_database(app)
}

fn save_resource_database(app: &AppHandle, database: &ResourceDatabase) -> Result<()> {
    let path = database_path(app)?;
    let text = serde_json::to_string_pretty(database)?;
    fs::write(path, text)?;
    Ok(())
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
