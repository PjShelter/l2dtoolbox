use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::jsonl::{parse_composite_jsonl, resolve_model_path};
use crate::types::{
    BatchAddReport, ModelCleanupReport, ModelInitParam, ModelJsonDocument, ModelPartOpacity,
    MtnPatchReport, PartsPresetMap, PresetApplyPayload, PresetApplyReport, PresetTarget,
    SelectorCopyPayload, SelectorCopyReport,
};

const PARTS_JSON_TEXT: &str = include_str!("../resources/parts.json");

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

pub fn read_model_json(file_path: &str) -> Result<ModelJsonDocument> {
    let text = fs::read_to_string(file_path)?;
    let data = serde_json::from_str::<Value>(&text)?;

    Ok(ModelJsonDocument {
        file_path: file_path.to_string(),
        model_version: detect_model_version(&data),
        init_opacities: extract_init_opacities(&data),
        init_params: extract_init_params(&data),
        motions: extract_motion_names(&data),
        expressions: extract_expression_names(&data),
        data,
    })
}

pub fn write_model_init_state(
    file_path: &str,
    init_opacities: Vec<ModelPartOpacity>,
    init_params: Vec<ModelInitParam>,
) -> Result<usize> {
    let text = fs::read_to_string(file_path)?;
    let mut data = serde_json::from_str::<Value>(&text)?;
    if !is_cubism2_value(&data) {
        anyhow::bail!("Current editor only supports Cubism 2 model.json files.");
    }

    data["init_opacities"] = Value::Array(
        init_opacities
            .into_iter()
            .map(|item| json!({ "id": item.id, "value": item.value }))
            .collect(),
    );
    data["init_params"] = Value::Array(
        init_params
            .into_iter()
            .map(|item| json!({ "id": item.id, "value": item.value }))
            .collect(),
    );

    let output = serde_json::to_string_pretty(&data)?;
    fs::write(file_path, &output)?;
    Ok(output.as_bytes().len())
}

pub fn read_parts_presets() -> Result<PartsPresetMap> {
    Ok(serde_json::from_str::<PartsPresetMap>(PARTS_JSON_TEXT)?)
}

pub fn scan_preset_targets(root_dir: &str, max_depth: Option<usize>) -> Result<Vec<PresetTarget>> {
    let presets = read_parts_presets()?;
    let max_depth = max_depth.unwrap_or(2);
    let root_path = Path::new(root_dir);
    let mut targets = Vec::new();

    for entry in WalkDir::new(root_path)
        .min_depth(1)
        .max_depth(max_depth + 1)
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if !path.is_file() || !is_cubism2_model_json(path) {
            continue;
        }

        let text = fs::read_to_string(path)?;
        let data = serde_json::from_str::<Value>(&text)?;
        targets.push(PresetTarget {
            model_path: path.to_string_lossy().to_string(),
            relative_path: pathdiff(path, root_path),
            detected_preset: detect_preset_name(&data, &presets),
        });
    }

    targets.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(targets)
}

pub fn apply_opacity_presets(payload: PresetApplyPayload) -> Result<PresetApplyReport> {
    let mut updated_models = Vec::new();
    let mut warnings = Vec::new();

    for row in payload.rows.iter().filter(|row| row.checked) {
        if row.preset_name == "保持不变" && row.init_opacities.is_none() {
            continue;
        }

        let Some(init_opacities) = row.init_opacities.as_ref() else {
            warnings.push(format!(
                "Skipped {} because no resolved init_opacities were provided.",
                row.model_path
            ));
            continue;
        };

        let text = fs::read_to_string(&row.model_path)?;
        let mut data = serde_json::from_str::<Value>(&text)?;
        data["init_opacities"] = Value::Array(
            init_opacities
                .iter()
                .map(|item| json!({ "id": item.id, "value": item.value }))
                .collect(),
        );
        let output = serde_json::to_string_pretty(&data)?;
        fs::write(&row.model_path, output)?;
        updated_models.push(row.model_path.clone());
    }

    let mut exported_assets = Vec::new();
    let mut skipped_assets = Vec::new();
    collect_selector_assets(
        &payload.root_dir,
        &payload.source_scope.mode,
        payload.source_scope.subdir.as_deref(),
        &payload.file_move_mode,
        &mut exported_assets,
        &mut skipped_assets,
        &mut warnings,
    )?;

    Ok(PresetApplyReport {
        updated_models,
        exported_assets,
        skipped_assets,
        warnings,
    })
}

pub fn copy_model_selectors(payload: SelectorCopyPayload) -> Result<SelectorCopyReport> {
    let source_text = fs::read_to_string(&payload.source_model_path)?;
    let source = serde_json::from_str::<Value>(&source_text)?;
    let mut updated_models = Vec::new();
    let mut warnings = Vec::new();

    for target_path in payload.target_model_paths {
        if target_path == payload.source_model_path {
            continue;
        }

        let text = match fs::read_to_string(&target_path) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("Failed to read {}: {}", target_path, error));
                continue;
            }
        };
        let mut target = match serde_json::from_str::<Value>(&text) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("Failed to parse {}: {}", target_path, error));
                continue;
            }
        };

        for field in &payload.fields {
            match field.as_str() {
                "motions" => apply_selector_field(&source, &mut target, "motions", &payload.mode),
                "expressions" => {
                    apply_selector_field(&source, &mut target, "expressions", &payload.mode)
                }
                _ => warnings.push(format!("Unsupported selector field: {}", field)),
            }
        }

        let output = serde_json::to_string_pretty(&target)?;
        fs::write(&target_path, output)?;
        updated_models.push(target_path);
    }

    Ok(SelectorCopyReport {
        updated_models,
        warnings,
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

fn detect_model_version(data: &Value) -> String {
    if is_cubism2_value(data) {
        return "cubism2".to_string();
    }
    if data
        .get("Version")
        .and_then(Value::as_i64)
        .map(|version| version >= 3)
        .unwrap_or(false)
        || data.get("FileReferences").and_then(Value::as_object).is_some()
    {
        return "cubism4".to_string();
    }
    "unknown".to_string()
}

fn is_cubism2_model_json(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if !name.eq_ignore_ascii_case("model.json") {
        return false;
    }
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let Ok(data) = serde_json::from_str::<Value>(&text) else {
        return false;
    };
    is_cubism2_value(&data)
}

fn is_cubism2_value(data: &Value) -> bool {
    ["version", "layout", "model"]
        .iter()
        .all(|key| data.get(*key).is_some())
}

fn extract_init_opacities(data: &Value) -> Vec<ModelPartOpacity> {
    data.get("init_opacities")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(|item| {
                    Some(ModelPartOpacity {
                        id: item.get("id")?.as_str()?.to_string(),
                        value: to_f64(item.get("value")?)?,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn extract_init_params(data: &Value) -> Vec<ModelInitParam> {
    data.get("init_params")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(|item| {
                    Some(ModelInitParam {
                        id: item.get("id")?.as_str()?.to_string(),
                        value: to_f64(item.get("value")?)?,
                        default_value: None,
                        min_value: None,
                        max_value: None,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn extract_motion_names(data: &Value) -> Vec<String> {
    data.get("motions")
        .and_then(Value::as_object)
        .map(|items| items.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default()
}

fn extract_expression_names(data: &Value) -> Vec<String> {
    data.get("expressions")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(|item| {
                    item.get("name")
                        .and_then(Value::as_str)
                        .map(|value| value.to_string())
                        .or_else(|| item.as_str().map(|value| value.to_string()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn detect_preset_name(data: &Value, presets: &PartsPresetMap) -> String {
    let visible_parts = data
        .get("init_opacities")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(|item| {
                    let value = to_f64(item.get("value")?)?;
                    if (value - 1.0).abs() > 0.000_001 {
                        return None;
                    }
                    item.get("id").and_then(Value::as_str).map(str::to_string)
                })
                .collect::<BTreeSet<_>>()
        });

    let Some(visible_parts) = visible_parts else {
        return "无".to_string();
    };

    for (preset_name, part_ids) in presets {
        let expected = part_ids.iter().cloned().collect::<BTreeSet<_>>();
        if expected == visible_parts {
            return preset_name.clone();
        }
    }

    if visible_parts.is_empty() {
        return "清空(全0)".to_string();
    }

    "自定义".to_string()
}

fn collect_selector_assets(
    root_dir: &str,
    scope_mode: &str,
    scope_subdir: Option<&str>,
    file_move_mode: &str,
    exported_assets: &mut Vec<String>,
    skipped_assets: &mut Vec<String>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let root_path = Path::new(root_dir);
    match scope_mode {
        "none" | "" => return Ok(()),
        "all" => {
            for entry in WalkDir::new(root_path).into_iter().flatten() {
                let path = entry.path();
                if !path.is_file() || !is_supported_asset(path) {
                    continue;
                }
                if path.components().any(|component| component.as_os_str() == "expnmtn") {
                    continue;
                }
                match export_selector_asset(root_path, path, None, file_move_mode) {
                    Ok(output) => exported_assets.push(output),
                    Err(error) => {
                        skipped_assets.push(path.to_string_lossy().to_string());
                        warnings.push(error.to_string());
                    }
                }
            }
        }
        "subdir" => {
            let Some(subdir) = scope_subdir else {
                warnings.push("Preset source scope mode 'subdir' requires a subdir.".to_string());
                return Ok(());
            };
            let source_base = root_path.join(subdir);
            if !source_base.is_dir() {
                warnings.push(format!(
                    "Preset source subdir does not exist: {}",
                    source_base.to_string_lossy()
                ));
                return Ok(());
            }
            for entry in WalkDir::new(&source_base).into_iter().flatten() {
                let path = entry.path();
                if !path.is_file() || !is_supported_asset(path) {
                    continue;
                }
                match export_selector_asset(root_path, path, Some(subdir), file_move_mode) {
                    Ok(output) => exported_assets.push(output),
                    Err(error) => {
                        skipped_assets.push(path.to_string_lossy().to_string());
                        warnings.push(error.to_string());
                    }
                }
            }
        }
        other => warnings.push(format!("Unknown preset source scope mode: {}", other)),
    }

    Ok(())
}

fn export_selector_asset(
    root_dir: &Path,
    source_path: &Path,
    fixed_group: Option<&str>,
    file_move_mode: &str,
) -> Result<String> {
    let relative_dir = source_path
        .parent()
        .and_then(|path| path.strip_prefix(root_dir).ok())
        .unwrap_or_else(|| Path::new(""));
    let top_level = fixed_group
        .map(|value| value.to_string())
        .or_else(|| {
            relative_dir
                .components()
                .next()
                .map(|component| component.as_os_str().to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "_root".to_string());

    let export_dir = root_dir.join("expnmtn").join(top_level);
    fs::create_dir_all(&export_dir)?;
    let file_name = source_path
        .file_name()
        .context("asset file has no file name")?;
    let destination = dedupe_target_path(&export_dir.join(file_name));

    if file_move_mode.eq_ignore_ascii_case("move") {
        move_or_copy(source_path, &destination, true)?;
    } else {
        move_or_copy(source_path, &destination, false)?;
    }

    Ok(destination.to_string_lossy().to_string())
}

fn move_or_copy(source: &Path, destination: &Path, remove_source: bool) -> Result<()> {
    if remove_source {
        match fs::rename(source, destination) {
            Ok(_) => return Ok(()),
            Err(_) => {
                fs::copy(source, destination)?;
                fs::remove_file(source)?;
                return Ok(());
            }
        }
    }

    fs::copy(source, destination)?;
    Ok(())
}

fn dedupe_target_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().and_then(|value| value.to_str()).unwrap_or("asset");
    let ext = path.extension().and_then(|value| value.to_str()).unwrap_or("");

    for index in 1.. {
        let candidate = if ext.is_empty() {
            parent.join(format!("{stem}_{index}"))
        } else {
            parent.join(format!("{stem}_{index}.{ext}"))
        };
        if !candidate.exists() {
            return candidate;
        }
    }

    path.to_path_buf()
}

fn apply_selector_field(source: &Value, target: &mut Value, field: &str, mode: &str) {
    match field {
        "motions" => {
            if mode.eq_ignore_ascii_case("overwrite") {
                if let Some(motions) = source.get(field) {
                    target[field] = motions.clone();
                } else {
                    target.as_object_mut().map(|object| object.remove(field));
                }
                return;
            }

            let merged = merge_motion_maps(source.get(field), target.get(field));
            if let Some(merged) = merged {
                target[field] = Value::Object(merged);
            }
        }
        "expressions" => {
            if mode.eq_ignore_ascii_case("overwrite") {
                if let Some(expressions) = source.get(field) {
                    target[field] = expressions.clone();
                } else {
                    target.as_object_mut().map(|object| object.remove(field));
                }
                return;
            }

            let merged = merge_expression_arrays(source.get(field), target.get(field));
            if !merged.is_empty() {
                target[field] = Value::Array(merged);
            }
        }
        _ => {}
    }
}

fn merge_motion_maps(
    source: Option<&Value>,
    target: Option<&Value>,
) -> Option<Map<String, Value>> {
    let mut merged = Map::new();

    for value in [target, source].into_iter().flatten() {
        let Some(object) = value.as_object() else {
            continue;
        };
        for (group_name, entries) in object {
            let bucket = merged
                .entry(group_name.clone())
                .or_insert_with(|| Value::Array(Vec::new()));
            let items = bucket.as_array_mut().expect("bucket must stay array");
            let mut seen = items.iter().map(Value::to_string).collect::<BTreeSet<_>>();
            if let Some(entries) = entries.as_array() {
                for entry in entries {
                    let key = entry.to_string();
                    if seen.insert(key) {
                        items.push(entry.clone());
                    }
                }
            }
        }
    }

    if merged.is_empty() {
        None
    } else {
        Some(merged)
    }
}

fn merge_expression_arrays(source: Option<&Value>, target: Option<&Value>) -> Vec<Value> {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();

    for value in [target, source].into_iter().flatten() {
        let Some(items) = value.as_array() else {
            continue;
        };
        for item in items {
            let key = item.to_string();
            if seen.insert(key) {
                merged.push(item.clone());
            }
        }
    }

    merged
}

fn to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
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
