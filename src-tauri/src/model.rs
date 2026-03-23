use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::jsonl::{parse_composite_jsonl, resolve_model_path};
use crate::types::{BatchAddReport, ModelCleanupReport, MtnPatchReport};

pub fn scan_model_directory(input_dir: &str) -> Result<Value> {
    let dir = Path::new(input_dir);
    let mut model = json!({
        "version": "Sample 1.0.0",
        "layout": { "center_x": 0, "center_y": 0, "width": 2 },
        "hit_areas_custom": {
            "head_x": [-0.25, 1],
            "head_y": [0.25, 0.2],
            "body_x": [-0.3, 0.2],
            "body_y": [0.3, -1.9]
        },
        "model": "",
        "textures": [],
        "motions": {},
        "expressions": []
    });

    let mut textures = Vec::new();
    let mut motions: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut expressions = Vec::new();

    for entry in WalkDir::new(dir).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let rel = pathdiff(path, dir);
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();

        if name.ends_with(".moc") {
            model["model"] = Value::String(rel);
        } else if name.ends_with(".physics.json") {
            model["physics"] = Value::String(rel);
        } else if name.ends_with(".png") {
            textures.push(Value::String(rel));
        } else if name.ends_with(".mtn") {
            let key = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or_default()
                .to_string();
            motions.entry(key).or_default().push(json!({ "file": rel }));
        } else if name.ends_with(".exp.json") {
            let exp_name = name.trim_end_matches(".exp.json").to_string();
            expressions.push(json!({ "name": exp_name, "file": rel }));
        }
    }

    model["textures"] = Value::Array(textures);
    model["motions"] = serde_json::to_value(motions)?;
    model["expressions"] = Value::Array(expressions);
    Ok(model)
}

pub fn clean_model_json(
    file_path: &str,
    skip_check: bool,
    auto_remove_missing: bool,
) -> Result<ModelCleanupReport> {
    let text = fs::read_to_string(file_path)?;
    let mut model = serde_json::from_str::<Value>(&text)?;
    let base_dir = Path::new(file_path)
        .parent()
        .context("model.json has no parent directory")?;

    let mut seen_motion_files = BTreeSet::new();
    let mut missing_files = Vec::new();
    let mut removed_motions = Vec::new();
    let mut removed_expressions = Vec::new();
    let mut cleaned_motions = Map::new();

    if let Some(motion_groups) = model.get("motions").and_then(Value::as_object) {
        for (motion_name, motion_entries) in motion_groups {
            let mut kept = Vec::new();
            if let Some(entries) = motion_entries.as_array() {
                for entry in entries.iter().rev() {
                    let Some(file) = entry.get("file").and_then(Value::as_str) else {
                        continue;
                    };
                    let abs_path = base_dir.join(file);
                    if seen_motion_files.contains(file) {
                        removed_motions.push(file.to_string());
                        continue;
                    }
                    if !skip_check && !abs_path.exists() {
                        missing_files.push(abs_path.to_string_lossy().to_string());
                        if auto_remove_missing {
                            removed_motions.push(file.to_string());
                            continue;
                        }
                    }
                    seen_motion_files.insert(file.to_string());
                    kept.insert(0, entry.clone());
                }
            }
            if !kept.is_empty() {
                cleaned_motions.insert(motion_name.clone(), Value::Array(kept));
            }
        }
    }

    let mut seen_expression_files = BTreeSet::new();
    let mut cleaned_expressions = Vec::new();
    if let Some(expressions) = model.get("expressions").and_then(Value::as_array) {
        for entry in expressions.iter().rev() {
            let Some(file) = entry.get("file").and_then(Value::as_str) else {
                continue;
            };
            let abs_path = base_dir.join(file);
            if seen_expression_files.contains(file) {
                removed_expressions.push(file.to_string());
                continue;
            }
            if !skip_check && !abs_path.exists() {
                missing_files.push(abs_path.to_string_lossy().to_string());
                if auto_remove_missing {
                    removed_expressions.push(file.to_string());
                    continue;
                }
            }
            seen_expression_files.insert(file.to_string());
            cleaned_expressions.insert(0, entry.clone());
        }
    }

    model["motions"] = Value::Object(cleaned_motions);
    model["expressions"] = Value::Array(cleaned_expressions);
    let updated = serde_json::to_string_pretty(&model)?;
    let changed = updated != text;
    if changed {
        fs::write(file_path, updated)?;
    }

    Ok(ModelCleanupReport {
        file_path: file_path.to_string(),
        removed_motions,
        removed_expressions,
        missing_files,
        changed,
    })
}

pub fn batch_add_assets(
    target_path: &str,
    asset_source: &str,
    prefix: Option<&str>,
) -> Result<BatchAddReport> {
    let prefix = prefix.unwrap_or_default();
    let asset_files = collect_asset_files(asset_source)?;
    let mut updated_files = Vec::new();
    let mut skipped_files = Vec::new();
    let mut added_count = 0usize;

    if target_path.ends_with(".jsonl") {
        let text = fs::read_to_string(target_path)?;
        let manifest = parse_composite_jsonl(&text, Some(target_path.to_string()));
        let jsonl_dir = Path::new(target_path)
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        for part in manifest.parts {
            let Some(model_path) = resolve_model_path(&jsonl_dir, &part.path) else {
                skipped_files.push(part.path.clone());
                continue;
            };
            added_count += update_model_json(&model_path, &asset_files, prefix)?;
            updated_files.push(model_path.to_string_lossy().to_string());
        }
    } else {
        let path = PathBuf::from(target_path);
        added_count += update_model_json(&path, &asset_files, prefix)?;
        updated_files.push(path.to_string_lossy().to_string());
    }

    updated_files.sort();
    updated_files.dedup();

    Ok(BatchAddReport {
        target_path: target_path.to_string(),
        updated_files,
        skipped_files,
        added_count,
    })
}

pub fn patch_mtn_param(dir_path: &str, param_name: &str, value: &str) -> Result<MtnPatchReport> {
    let mut updated_files = Vec::new();
    let mut added_files = Vec::new();
    for entry in WalkDir::new(dir_path).into_iter().flatten() {
        let path = entry.path();
        if !is_mtn(path) {
            continue;
        }
        let text = fs::read_to_string(path)?;
        let mut found = false;
        let mut next_lines = Vec::new();
        for line in text.lines() {
            if line.trim_start().starts_with(&format!("{param_name}=")) {
                next_lines.push(format!("{param_name}={value}"));
                found = true;
            } else {
                next_lines.push(line.to_string());
            }
        }
        if found {
            updated_files.push(path.to_string_lossy().to_string());
        } else {
            next_lines.push(format!("{param_name}={value}"));
            added_files.push(path.to_string_lossy().to_string());
        }
        fs::write(path, next_lines.join("\n"))?;
    }

    Ok(MtnPatchReport {
        dir_path: dir_path.to_string(),
        param_name: param_name.to_string(),
        updated_files,
        added_files,
        removed_files: Vec::new(),
    })
}

pub fn remove_mtn_param(dir_path: &str, param_name: &str) -> Result<MtnPatchReport> {
    let mut removed_files = Vec::new();
    for entry in WalkDir::new(dir_path).into_iter().flatten() {
        let path = entry.path();
        if !is_mtn(path) {
            continue;
        }
        let text = fs::read_to_string(path)?;
        let next_lines = text
            .lines()
            .filter(|line| !line.trim_start().starts_with(&format!("{param_name}=")))
            .map(|line| line.to_string())
            .collect::<Vec<_>>();
        if next_lines.join("\n") != text {
            removed_files.push(path.to_string_lossy().to_string());
            fs::write(path, next_lines.join("\n"))?;
        }
    }

    Ok(MtnPatchReport {
        dir_path: dir_path.to_string(),
        param_name: param_name.to_string(),
        updated_files: Vec::new(),
        added_files: Vec::new(),
        removed_files,
    })
}

fn update_model_json(model_path: &Path, asset_files: &[PathBuf], prefix: &str) -> Result<usize> {
    let text = fs::read_to_string(model_path)?;
    let mut model = serde_json::from_str::<Value>(&text)?;
    let base_dir = model_path.parent().context("model.json missing parent")?;
    ensure_object(&mut model, "motions");
    ensure_array(&mut model, "expressions");

    let mut added = 0usize;
    for asset_file in asset_files {
        if !asset_file.exists() {
            continue;
        }
        let rel = pathdiff(asset_file, base_dir);
        let file_name = asset_file.file_name().and_then(|name| name.to_str()).unwrap_or_default();

        if file_name.ends_with(".mtn") {
            let motion_name = format!(
                "{}{}",
                prefix,
                asset_file
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or_default()
            );
            let motions = model.get_mut("motions").and_then(Value::as_object_mut).unwrap();
            motions
                .entry(motion_name)
                .or_insert_with(|| Value::Array(Vec::new()))
                .as_array_mut()
                .unwrap()
                .push(json!({ "file": rel }));
            added += 1;
        } else if file_name.ends_with(".exp.json") {
            let exp_name = format!("{}{}", prefix, file_name.trim_end_matches(".exp.json"));
            model["expressions"]
                .as_array_mut()
                .unwrap()
                .push(json!({ "name": exp_name, "file": rel }));
            added += 1;
        }
    }

    if added > 0 {
        fs::write(model_path, serde_json::to_string_pretty(&model)?)?;
    }
    Ok(added)
}

fn collect_asset_files(asset_source: &str) -> Result<Vec<PathBuf>> {
    let source = asset_source.trim();
    if source.is_empty() {
        return Ok(Vec::new());
    }
    let source_path = Path::new(source);
    if source_path.is_dir() {
        let mut files = Vec::new();
        for entry in WalkDir::new(source_path).into_iter().flatten() {
            let path = entry.path();
            if path.is_file() && is_supported_asset(path) {
                files.push(path.to_path_buf());
            }
        }
        return Ok(files);
    }

    Ok(source
        .split(';')
        .map(|item| PathBuf::from(item.trim().trim_matches('"')))
        .collect())
}

fn is_supported_asset(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    file_name.ends_with(".mtn") || file_name.ends_with(".exp.json")
}

fn ensure_object(model: &mut Value, key: &str) {
    if model.get(key).and_then(Value::as_object).is_none() {
        model[key] = Value::Object(Map::new());
    }
}

fn ensure_array(model: &mut Value, key: &str) {
    if model.get(key).and_then(Value::as_array).is_none() {
        model[key] = Value::Array(Vec::new());
    }
}

fn is_mtn(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("mtn"))
            .unwrap_or(false)
}

fn pathdiff(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_asset_files_accepts_semicolon_list() {
        let files = collect_asset_files(r#"a.mtn;b.exp.json"#).unwrap();
        assert_eq!(files.len(), 2);
    }
}
