use anyhow::Result;
use serde_json::{Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::types::{
    CompositeDiagnostic, CompositeManifest, CompositePart, CompositeSummary,
    OptimizedCompositeModel, ResolvedCompositeManifest, ResolvedCompositePart,
};

const MEDIA_IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "avif", "bmp"];
const MEDIA_VIDEO_EXTS: &[&str] = &["webm", "mp4", "ogv", "mov", "mkv"];

pub fn read_jsonl(file_path: &str) -> Result<CompositeManifest> {
    let text = fs::read_to_string(file_path)?;
    Ok(parse_composite_jsonl(&text, Some(file_path.to_string())))
}

pub fn parse_composite_jsonl(text: &str, source: Option<String>) -> CompositeManifest {
    let mut diagnostics = Vec::new();
    let mut parts = Vec::new();
    let mut summary = CompositeSummary::default();

    for (index, raw_line) in text.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parsed = match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => value,
            Err(_) => {
                diagnostics.push(CompositeDiagnostic {
                    code: "invalid-json".to_string(),
                    message: "Line is not valid JSON.".to_string(),
                    severity: "warning".to_string(),
                    line_number: Some(line_number),
                    line: Some(raw_line.to_string()),
                    field: None,
                });
                continue;
            }
        };

        let Some(object) = parsed.as_object() else {
            diagnostics.push(CompositeDiagnostic {
                code: "invalid-root".to_string(),
                message: "Line must parse to a JSON object.".to_string(),
                severity: "warning".to_string(),
                line_number: Some(line_number),
                line: Some(raw_line.to_string()),
                field: None,
            });
            continue;
        };

        if is_summary_object(object) {
            merge_summary_object(&mut summary, object, line_number);
            continue;
        }

        let Some(path) = object.get("path").and_then(Value::as_str) else {
            diagnostics.push(CompositeDiagnostic {
                code: "missing-path".to_string(),
                message: "Part lines must include a valid path field.".to_string(),
                severity: "warning".to_string(),
                line_number: Some(line_number),
                line: Some(raw_line.to_string()),
                field: Some("path".to_string()),
            });
            continue;
        };

        parts.push(parse_part_object(path, object, line_number));
    }

    CompositeManifest {
        source,
        raw_text: text.to_string(),
        parts,
        summary: clean_summary(summary),
        diagnostics,
    }
}

pub fn optimize_jsonl(manifest: CompositeManifest) -> OptimizedCompositeModel {
    let mut parts = manifest.parts.clone();
    for (index, part) in parts.iter_mut().enumerate() {
        if part.index.is_none() {
            part.index = Some(index as i64);
        }
    }

    let mut summary = clean_summary(manifest.summary.clone());
    if summary.version.is_none() && requires_version_two(&parts) {
        summary.version = Some(2);
    }

    let text = stringify_composite_jsonl(&parts, &summary);
    OptimizedCompositeModel {
        manifest: CompositeManifest {
            source: manifest.source.clone(),
            raw_text: text.clone(),
            parts,
            summary,
            diagnostics: manifest.diagnostics,
        },
        changed: text != manifest.raw_text,
        text,
    }
}

pub fn stringify_composite_jsonl(parts: &[CompositePart], summary: &CompositeSummary) -> String {
    let mut lines = Vec::new();
    for part in parts {
        let mut object = Map::new();
        object.insert("path".to_string(), Value::String(normalize_slashes(&part.path)));
        if let Some(part_type) = &part.part_type {
            object.insert("type".to_string(), Value::String(part_type.clone()));
        }
        insert_string(&mut object, "id", &part.id);
        insert_string(&mut object, "folder", &part.folder);
        insert_number_i64(&mut object, "index", part.index);
        insert_number_f64(&mut object, "x", part.x);
        insert_number_f64(&mut object, "y", part.y);
        insert_number_f64(&mut object, "xscale", part.xscale);
        insert_number_f64(&mut object, "yscale", part.yscale);
        insert_bool(&mut object, "loop", part.loop_flag);
        insert_bool(&mut object, "muted", part.muted);
        insert_bool(&mut object, "autoplay", part.autoplay);
        insert_bool(&mut object, "playsinline", part.playsinline);
        lines.push(Value::Object(object).to_string());
    }

    let summary = clean_summary(summary.clone());
    let mut summary_map = Map::new();
    insert_number_i64(&mut summary_map, "version", summary.version);
    insert_string_array(&mut summary_map, "motions", summary.motions.as_ref());
    insert_string_array(
        &mut summary_map,
        "expressions",
        summary.expressions.as_ref(),
    );
    insert_number_f64(&mut summary_map, "import", summary.import);
    if !summary_map.is_empty() {
        lines.push(Value::Object(summary_map).to_string());
    }

    lines.join("\n")
}

pub fn write_jsonl(file_path: &str, manifest: CompositeManifest) -> Result<usize> {
    let text = stringify_composite_jsonl(&manifest.parts, &manifest.summary);
    fs::write(file_path, &text)?;
    Ok(text.as_bytes().len())
}

pub fn resolve_preview_assets(
    source_path: &str,
    manifest: CompositeManifest,
) -> Result<ResolvedCompositeManifest> {
    let source_dir = Path::new(source_path)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let game_root = find_game_root(&source_dir);

    let mut resolved_parts = Vec::new();
    for part in manifest.parts.iter() {
        let resolved_path = resolve_part_path(&source_dir, &game_root, part)?;
        resolved_parts.push(ResolvedCompositePart {
            part: part.clone(),
            resolved_path: resolved_path.to_string_lossy().to_string(),
        });
    }

    Ok(ResolvedCompositeManifest {
        source: manifest.source,
        raw_text: manifest.raw_text,
        parts: resolved_parts,
        summary: manifest.summary,
        diagnostics: manifest.diagnostics,
    })
}

pub fn resolve_model_path(jsonl_dir: &Path, raw_path: &str) -> Option<PathBuf> {
    resolve_generic_path(jsonl_dir, raw_path, true)
}

fn resolve_part_path(source_dir: &Path, game_root: &Path, part: &CompositePart) -> Result<PathBuf> {
    let part_type = infer_part_type(part);
    let resolved = if part_type == "live2d" {
        resolve_generic_path(source_dir, &part.path, true)
    } else {
        resolve_generic_path_with_root(source_dir, game_root, &part.path, false)
    };

    Ok(resolved.unwrap_or_else(|| source_dir.join(part.path.clone())))
}

fn resolve_generic_path(jsonl_dir: &Path, raw_path: &str, model_only: bool) -> Option<PathBuf> {
    let game_root = find_game_root(jsonl_dir);
    resolve_generic_path_with_root(jsonl_dir, &game_root, raw_path, model_only)
}

fn resolve_generic_path_with_root(
    source_dir: &Path,
    game_root: &Path,
    raw_path: &str,
    model_only: bool,
) -> Option<PathBuf> {
    let raw_path = raw_path.trim();
    if raw_path.is_empty() {
        return None;
    }

    let as_path = PathBuf::from(raw_path);
    if as_path.is_absolute() && as_path.exists() {
        return Some(as_path);
    }

    let mut rel = normalize_slashes(raw_path);
    if rel.starts_with("game/") {
        rel = rel.trim_start_matches("game/").to_string();
    }

    let direct = game_root.join(&rel);
    if direct.exists() && (!model_only || is_model_json(&direct)) {
        return Some(direct);
    }

    let figure = game_root.join("figure").join(&rel);
    if figure.exists() && (!model_only || is_model_json(&figure)) {
        return Some(figure);
    }

    let local = source_dir.join(&rel);
    if local.exists() && (!model_only || is_model_json(&local)) {
        return Some(local);
    }

    find_path_by_tail(game_root, &rel, model_only)
}

fn find_path_by_tail(root: &Path, rel: &str, model_only: bool) -> Option<PathBuf> {
    let tail = rel.replace('/', &std::path::MAIN_SEPARATOR.to_string());
    for entry in WalkDir::new(root).into_iter().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if model_only && !is_model_json(path) {
            continue;
        }
        let path_text = path.to_string_lossy().to_lowercase();
        if path_text.ends_with(&tail.to_lowercase()) {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn find_game_root(start_dir: &Path) -> PathBuf {
    let mut current = start_dir.to_path_buf();
    loop {
        if current
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.eq_ignore_ascii_case("game"))
            .unwrap_or(false)
        {
            return current;
        }
        let Some(parent) = current.parent() else {
            return start_dir.to_path_buf();
        };
        current = parent.to_path_buf();
    }
}

fn parse_part_object(path: &str, object: &Map<String, Value>, line_number: usize) -> CompositePart {
    CompositePart {
        path: normalize_slashes(path),
        part_type: object.get("type").and_then(Value::as_str).map(|value| value.to_string()),
        id: object.get("id").and_then(Value::as_str).map(|value| value.to_string()),
        folder: object
            .get("folder")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        index: object.get("index").and_then(to_i64),
        x: object.get("x").and_then(to_f64),
        y: object.get("y").and_then(to_f64),
        xscale: object.get("xscale").and_then(to_f64),
        yscale: object.get("yscale").and_then(to_f64),
        loop_flag: object.get("loop").and_then(Value::as_bool),
        muted: object.get("muted").and_then(Value::as_bool),
        autoplay: object.get("autoplay").and_then(Value::as_bool),
        playsinline: object.get("playsinline").and_then(Value::as_bool),
        line_number: Some(line_number),
    }
}

fn is_summary_object(object: &Map<String, Value>) -> bool {
    !object.contains_key("path")
        && ["version", "motions", "expressions", "import"]
            .iter()
            .any(|field| object.contains_key(*field))
}

fn merge_summary_object(summary: &mut CompositeSummary, object: &Map<String, Value>, line_number: usize) {
    if let Some(version) = object.get("version").and_then(to_i64) {
        summary.version = Some(version);
    }
    if let Some(motions) = object.get("motions").and_then(to_string_vec) {
        summary.motions = Some(merge_string_arrays(summary.motions.take(), motions));
    }
    if let Some(expressions) = object.get("expressions").and_then(to_string_vec) {
        summary.expressions = Some(merge_string_arrays(summary.expressions.take(), expressions));
    }
    if let Some(import) = object.get("import").and_then(to_f64) {
        summary.import = Some(import);
    }
    summary.line_number = Some(line_number);
}

fn clean_summary(mut summary: CompositeSummary) -> CompositeSummary {
    summary.motions = summary.motions.take().map(dedupe_strings).filter(|items| !items.is_empty());
    summary.expressions = summary
        .expressions
        .take()
        .map(dedupe_strings)
        .filter(|items| !items.is_empty());
    if let Some(version) = summary.version {
        if version < 1 {
            summary.version = None;
        }
    }
    summary
}

fn requires_version_two(parts: &[CompositePart]) -> bool {
    parts.iter().any(|part| {
        part.part_type.is_some()
            || part.loop_flag.is_some()
            || part.muted.is_some()
            || part.autoplay.is_some()
            || part.playsinline.is_some()
    })
}

fn infer_part_type(part: &CompositePart) -> String {
    if let Some(part_type) = &part.part_type {
        return part_type.clone();
    }
    let ext = Path::new(&part.path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if MEDIA_IMAGE_EXTS.contains(&ext.as_str()) {
        return "image".to_string();
    }
    if ext == "gif" {
        return "gif".to_string();
    }
    if MEDIA_VIDEO_EXTS.contains(&ext.as_str()) {
        return "video".to_string();
    }
    "live2d".to_string()
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/").trim().trim_matches('"').to_string()
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut deduped = Vec::new();
    for value in values {
        if !value.trim().is_empty() && !deduped.iter().any(|item| item == &value) {
            deduped.push(value);
        }
    }
    deduped
}

fn merge_string_arrays(existing: Option<Vec<String>>, next: Vec<String>) -> Vec<String> {
    let mut merged = existing.unwrap_or_default();
    merged.extend(next);
    dedupe_strings(merged)
}

fn to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64().or_else(|| number.as_f64().map(|item| item as i64)),
        Value::String(text) => text.parse::<f64>().ok().map(|item| item as i64),
        _ => None,
    }
}

fn to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
}

fn to_string_vec(value: &Value) -> Option<Vec<String>> {
    value.as_array().map(|items| {
        items
            .iter()
            .filter_map(Value::as_str)
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    })
}

fn insert_string(map: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value.clone()));
    }
}

fn insert_string_array(map: &mut Map<String, Value>, key: &str, value: Option<&Vec<String>>) {
    if let Some(items) = value {
        map.insert(
            key.to_string(),
            Value::Array(items.iter().map(|item| Value::String(item.clone())).collect()),
        );
    }
}

fn insert_bool(map: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::Bool(value));
    }
}

fn insert_number_i64(map: &mut Map<String, Value>, key: &str, value: Option<i64>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::from(value));
    }
}

fn insert_number_f64(map: &mut Map<String, Value>, key: &str, value: Option<f64>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::from(value));
    }
}

fn is_model_json(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.eq_ignore_ascii_case("model.json") || name.ends_with(".model3.json"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_jsonl_extracts_parts_and_summary() {
        let manifest = parse_composite_jsonl(
            "{\"path\":\"body/model.json\"}\n{\"motions\":[\"idle01\"],\"import\":50}",
            Some("demo".to_string()),
        );

        assert_eq!(manifest.parts.len(), 1);
        assert_eq!(manifest.summary.motions.unwrap(), vec!["idle01".to_string()]);
        assert_eq!(manifest.summary.import, Some(50.0));
    }
}
