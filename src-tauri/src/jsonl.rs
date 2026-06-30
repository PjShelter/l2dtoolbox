use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::types::{
    CompositeDiagnostic, CompositeManifest, CompositePart, CompositeSummary,
    ConversionReport, ConversionScannedSelectors, GeneratedJsonl, JsonlGenerationPayload,
    OptimizedCompositeModel, ResolvedCompositeManifest, ResolvedCompositePart,
};

const MEDIA_IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "avif", "bmp"];
const MEDIA_VIDEO_EXTS: &[&str] = &["webm", "mp4", "ogv", "mov", "mkv"];
const FLOAT_EPSILON: f64 = 0.000_001;
const DEFAULT_JSONL_VERSION: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WmdlDocument {
    name: String,
    model_relative_path: String,
    figure_template: String,
    transform_template: String,
    #[serde(default)]
    sub_models: Vec<WmdlSubModel>,
    #[serde(default)]
    x: f64,
    #[serde(default)]
    y: f64,
    #[serde(default = "default_scale")]
    scale: f64,
    #[serde(default)]
    rotation: f64,
    #[serde(default)]
    reverse_x: bool,
    #[serde(default)]
    live2d_bounds: [f64; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WmdlSubModel {
    model_relative_path: String,
    #[serde(default)]
    offset_x: f64,
    #[serde(default)]
    offset_y: f64,
}

fn default_scale() -> f64 {
    1.0
}

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
    if requires_version_two(&parts) && summary.version.unwrap_or(DEFAULT_JSONL_VERSION) < 2 {
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

pub fn generate_jsonl_from_selection(payload: &JsonlGenerationPayload) -> Result<GeneratedJsonl> {
    if payload.selected_relative_paths.is_empty() {
        anyhow::bail!("At least one model must be selected.");
    }

    let root_dir = Path::new(&payload.root_dir);
    let root_name = root_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("model");

    let mut parts = Vec::new();
    let mut common_motions: Option<BTreeSet<String>> = None;
    let mut expressions = Vec::new();

    for (index, relative_path) in payload.selected_relative_paths.iter().enumerate() {
        let normalized_relative = normalize_slashes(relative_path);
        let absolute_path = root_dir.join(relative_path);
        if !absolute_path.is_file() {
            anyhow::bail!("Model file does not exist: {}", absolute_path.to_string_lossy());
        }
        if !is_model_json(&absolute_path) {
            anyhow::bail!(
                "Selected file is not a supported model settings file: {}",
                absolute_path.to_string_lossy()
            );
        }

        let selectors = read_model_selectors_from_path(&absolute_path)?;
        let motion_set = selectors.motions.into_iter().collect::<BTreeSet<_>>();
        common_motions = Some(match common_motions.take() {
            Some(existing) => existing
                .intersection(&motion_set)
                .cloned()
                .collect::<BTreeSet<_>>(),
            None => motion_set,
        });

        for expression in selectors.expressions {
            if !expressions.iter().any(|item| item == &expression) {
                expressions.push(expression);
            }
        }

        let folder = Path::new(&normalized_relative)
            .parent()
            .and_then(|path| path.to_str())
            .map(normalize_slashes)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| ".".to_string());

        parts.push(CompositePart {
            path: normalized_relative,
            part_type: None,
            id: Some(format!("{}{}", payload.id_prefix, index)),
            folder: Some(folder),
            index: Some(index as i64),
            x: None,
            y: None,
            xscale: None,
            yscale: None,
            loop_flag: None,
            muted: None,
            autoplay: None,
            playsinline: None,
            line_number: None,
        });
    }

    let manifest = CompositeManifest {
        source: None,
        raw_text: String::new(),
        parts,
        summary: CompositeSummary {
            version: Some(DEFAULT_JSONL_VERSION),
            motions: common_motions.map(|items| items.into_iter().collect()),
            expressions: Some(expressions).filter(|items| !items.is_empty()),
            import: payload.summary_import,
            line_number: None,
        },
        diagnostics: Vec::new(),
    };
    let optimized = optimize_jsonl(manifest);

    Ok(GeneratedJsonl {
        manifest: optimized.manifest,
        text: optimized.text,
        suggested_file_name: format!("{root_name}.jsonl"),
        selected_count: payload.selected_relative_paths.len(),
    })
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

pub fn jsonl_to_wmdl(file_path: &str) -> Result<ConversionReport> {
    let manifest = read_jsonl(file_path)?;
    let file_stem = Path::new(file_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("model")
        .to_string();

    let mut warnings = Vec::new();
    let live2d_parts = manifest
        .parts
        .iter()
        .filter_map(|part| {
            let part_type = infer_part_type(part);
            if part_type != "live2d" {
                warnings.push(format!("Skipped non-live2d part: {}", part.path));
                return None;
            }
            Some(part.clone())
        })
        .collect::<Vec<_>>();

    if live2d_parts.is_empty() {
        anyhow::bail!("JSONL does not contain any live2d parts.");
    }

    let main_part = live2d_parts[0].clone();
    let main_x = main_part.x.unwrap_or(0.0);
    let main_y = main_part.y.unwrap_or(0.0);
    let scale = choose_uniform_scale(&main_part, &mut warnings);

    let sub_models = live2d_parts
        .iter()
        .skip(1)
        .map(|part| {
            if !is_uniform_scale(part) {
                warnings.push(format!(
                    "Part {} uses non-uniform scaling; WMDL only keeps a single scale.",
                    part.path
                ));
            }
            WmdlSubModel {
                model_relative_path: normalize_slashes(&part.path),
                offset_x: part.x.unwrap_or(0.0) - main_x,
                offset_y: part.y.unwrap_or(0.0) - main_y,
            }
        })
        .collect::<Vec<_>>();

    let wmdl = WmdlDocument {
        name: file_stem.clone(),
        model_relative_path: normalize_slashes(&main_part.path),
        figure_template: format!("changeFigure:%conf_path% -id={file_stem}_0 -zIndex=0 %me_0%;"),
        transform_template: format!("setTransform:%me_0% -target={file_stem}_0 -duration=750 -writeDefault;"),
        sub_models,
        x: main_x,
        y: main_y,
        scale,
        rotation: 0.0,
        reverse_x: false,
        live2d_bounds: [0.0, 0.0, 0.0, 0.0],
    };

    let output_path = Path::new(file_path)
        .with_extension("wmdl")
        .to_string_lossy()
        .to_string();
    let output_text = serde_json::to_string_pretty(&wmdl)?;
    fs::write(&output_path, output_text)?;

    Ok(ConversionReport {
        input_path: file_path.to_string(),
        output_path,
        warnings,
        scanned_selectors: ConversionScannedSelectors {
            motions: manifest.summary.motions.unwrap_or_default(),
            expressions: manifest.summary.expressions.unwrap_or_default(),
        },
    })
}

pub fn read_wmdl(file_path: &str) -> Result<CompositeManifest> {
    let (text, _warnings, _scanned) = convert_wmdl_to_jsonl_text(file_path, None)?;
    Ok(parse_composite_jsonl(&text, Some(file_path.to_string())))
}

pub fn wmdl_to_jsonl(file_path: &str, figure_root_dir: Option<&str>) -> Result<ConversionReport> {
    let (text, warnings, scanned_selectors) =
        convert_wmdl_to_jsonl_text(file_path, figure_root_dir)?;

    let output_path = Path::new(file_path)
        .with_extension("jsonl")
        .to_string_lossy()
        .to_string();
    fs::write(&output_path, format!("{}\n", text))?;

    Ok(ConversionReport {
        input_path: file_path.to_string(),
        output_path,
        warnings,
        scanned_selectors,
    })
}

fn convert_wmdl_to_jsonl_text(
    file_path: &str,
    figure_root_dir: Option<&str>,
) -> Result<(String, Vec<String>, ConversionScannedSelectors)> {
    let text = fs::read_to_string(file_path)?;
    let wmdl = serde_json::from_str::<WmdlDocument>(&text)?;

    let mut parts = Vec::new();
    let main_folder = dirname_text(&wmdl.model_relative_path);
    parts.push(CompositePart {
        path: normalize_slashes(&wmdl.model_relative_path),
        part_type: None,
        id: Some("dao0".to_string()),
        folder: Some(main_folder),
        index: Some(0),
        x: Some(wmdl.x),
        y: Some(wmdl.y),
        xscale: Some(wmdl.scale),
        yscale: Some(wmdl.scale),
        loop_flag: None,
        muted: None,
        autoplay: None,
        playsinline: None,
        line_number: None,
    });

    for (index, sub_model) in wmdl.sub_models.iter().enumerate() {
        parts.push(CompositePart {
            path: normalize_slashes(&sub_model.model_relative_path),
            part_type: None,
            id: Some(format!("dao{}", index + 1)),
            folder: Some(dirname_text(&sub_model.model_relative_path)),
            index: Some((index + 1) as i64),
            x: Some(wmdl.x + sub_model.offset_x),
            y: Some(wmdl.y + sub_model.offset_y),
            xscale: Some(wmdl.scale),
            yscale: Some(wmdl.scale),
            loop_flag: None,
            muted: None,
            autoplay: None,
            playsinline: None,
            line_number: None,
        });
    }

    let figure_root = figure_root_dir
        .map(PathBuf::from)
        .or_else(|| Path::new(file_path).parent().map(PathBuf::from));

    let mut warnings = Vec::new();
    let scanned_selectors = scan_selectors_for_paths(
        figure_root,
        parts.iter().map(|part| part.path.clone()).collect(),
        &mut warnings,
    );

    let manifest = CompositeManifest {
        source: None,
        raw_text: String::new(),
        parts,
        summary: CompositeSummary {
            version: Some(DEFAULT_JSONL_VERSION),
            motions: Some(scanned_selectors.motions.clone()).filter(|items| !items.is_empty()),
            expressions: Some(scanned_selectors.expressions.clone())
                .filter(|items| !items.is_empty()),
            import: None,
            line_number: None,
        },
        diagnostics: Vec::new(),
    };
    let optimized = optimize_jsonl(manifest);

    Ok((optimized.text, warnings, scanned_selectors))
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

fn scan_selectors_for_paths(
    figure_root_dir: Option<PathBuf>,
    relative_paths: Vec<String>,
    warnings: &mut Vec<String>,
) -> ConversionScannedSelectors {
    let mut motions = Vec::new();
    let mut expressions = Vec::new();

    let Some(root_dir) = figure_root_dir else {
        return ConversionScannedSelectors { motions, expressions };
    };

    for relative_path in relative_paths {
        let candidate = root_dir.join(&relative_path);
        if !candidate.exists() {
            warnings.push(format!("Selector scan skipped missing model: {}", candidate.to_string_lossy()));
            continue;
        }
        match read_model_selectors_from_path(&candidate) {
            Ok(selectors) => {
                for motion in selectors.motions {
                    if !motions.iter().any(|item| item == &motion) {
                        motions.push(motion);
                    }
                }
                for expression in selectors.expressions {
                    if !expressions.iter().any(|item| item == &expression) {
                        expressions.push(expression);
                    }
                }
            }
            Err(error) => warnings.push(format!(
                "Selector scan failed for {}: {}",
                candidate.to_string_lossy(),
                error
            )),
        }
    }

    ConversionScannedSelectors { motions, expressions }
}

fn read_model_selectors_from_path(path: &Path) -> Result<ConversionScannedSelectors> {
    let text = fs::read_to_string(path)?;
    let value = serde_json::from_str::<Value>(&text)?;
    Ok(read_model_selectors_from_value(&value))
}

fn read_model_selectors_from_value(value: &Value) -> ConversionScannedSelectors {
    let motions = value
        .get("motions")
        .and_then(Value::as_object)
        .map(|items| items.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();

    let expressions = value
        .get("expressions")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(|item| {
                    item.get("name")
                        .and_then(Value::as_str)
                        .map(|text| text.trim().to_string())
                        .filter(|text| !text.is_empty())
                        .or_else(|| item.as_str().map(|text| text.trim().to_string()))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    ConversionScannedSelectors {
        motions: dedupe_strings(motions),
        expressions: dedupe_strings(expressions),
    }
}

fn choose_uniform_scale(part: &CompositePart, warnings: &mut Vec<String>) -> f64 {
    let xscale = part.xscale.unwrap_or(1.0);
    let yscale = part.yscale.unwrap_or(xscale);
    if (xscale - yscale).abs() > FLOAT_EPSILON {
        warnings.push(format!(
            "Part {} uses non-uniform scaling ({xscale}, {yscale}); WMDL keeps {xscale}.",
            part.path
        ));
    }
    xscale
}

fn is_uniform_scale(part: &CompositePart) -> bool {
    let xscale = part.xscale.unwrap_or(1.0);
    let yscale = part.yscale.unwrap_or(xscale);
    (xscale - yscale).abs() <= FLOAT_EPSILON
}

fn dirname_text(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|value| value.to_str())
        .map(normalize_slashes)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| ".".to_string())
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
    use std::time::SystemTime;

    fn unique_tmp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("l2d-wmdl-{label}-{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

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

    #[test]
    fn generate_jsonl_defaults_to_version_one() {
        let dir = unique_tmp_dir("gen-v1");
        let body = dir.join("1.body");
        let face = dir.join("2.face");
        fs::create_dir_all(&body).unwrap();
        fs::create_dir_all(&face).unwrap();
        fs::write(
            body.join("model.json"),
            "{\"version\":\"Sample 1.0.0\",\"layout\":{},\"model\":\"body.moc\",\
             \"motions\":{\"idle\":[{\"file\":\"idle.mtn\"}],\"wave\":[{\"file\":\"wave.mtn\"}]},\
             \"expressions\":[{\"name\":\"smile\",\"file\":\"smile.exp.json\"}]}",
        )
        .unwrap();
        fs::write(
            face.join("model.json"),
            "{\"version\":\"Sample 1.0.0\",\"layout\":{},\"model\":\"face.moc\",\
             \"motions\":{\"idle\":[{\"file\":\"idle.mtn\"}]},\
             \"expressions\":[{\"name\":\"blink\",\"file\":\"blink.exp.json\"}]}",
        )
        .unwrap();

        let generated = generate_jsonl_from_selection(&JsonlGenerationPayload {
            root_dir: dir.to_string_lossy().to_string(),
            selected_relative_paths: vec![
                "1.body/model.json".to_string(),
                "2.face/model.json".to_string(),
            ],
            id_prefix: "part".to_string(),
            summary_import: None,
        })
        .unwrap();

        assert_eq!(generated.manifest.summary.version, Some(1));
        assert!(generated.text.contains("\"version\":1"));
        assert!(generated.text.contains("\"motions\":[\"idle\"]"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn optimize_jsonl_upgrades_v1_when_v2_fields_are_present() {
        let manifest = CompositeManifest {
            source: None,
            raw_text: String::new(),
            parts: vec![CompositePart {
                path: "overlay.png".to_string(),
                part_type: Some("image".to_string()),
                id: None,
                folder: None,
                index: None,
                x: None,
                y: None,
                xscale: None,
                yscale: None,
                loop_flag: None,
                muted: None,
                autoplay: None,
                playsinline: None,
                line_number: None,
            }],
            summary: CompositeSummary {
                version: Some(1),
                motions: None,
                expressions: None,
                import: None,
                line_number: None,
            },
            diagnostics: Vec::new(),
        };

        let optimized = optimize_jsonl(manifest);
        assert_eq!(optimized.manifest.summary.version, Some(2));
        assert!(optimized.text.contains("\"version\":2"));
    }

    #[test]
    fn jsonl_to_wmdl_and_back_round_trips_paths_and_offsets() {
        let dir = unique_tmp_dir("rt");
        let jsonl_path = dir.join("demo.jsonl");
        fs::write(
            &jsonl_path,
            "{\"path\":\"model/1/main/model.json\",\"x\":10,\"y\":20,\"xscale\":1.5,\"yscale\":1.5}\n\
             {\"path\":\"model/2/sub/model.json\",\"x\":12,\"y\":25}\n\
             {\"path\":\"model/3/sub/model.json\",\"x\":8,\"y\":18}\n",
        )
        .unwrap();

        let report = jsonl_to_wmdl(jsonl_path.to_str().unwrap()).unwrap();
        let wmdl_text = fs::read_to_string(&report.output_path).unwrap();
        let wmdl: WmdlDocument = serde_json::from_str(&wmdl_text).unwrap();

        assert_eq!(wmdl.model_relative_path, "model/1/main/model.json");
        assert_eq!(wmdl.x, 10.0);
        assert_eq!(wmdl.y, 20.0);
        assert_eq!(wmdl.scale, 1.5);
        assert_eq!(wmdl.sub_models.len(), 2);
        assert_eq!(wmdl.sub_models[0].model_relative_path, "model/2/sub/model.json");
        assert_eq!(wmdl.sub_models[0].offset_x, 2.0);
        assert_eq!(wmdl.sub_models[0].offset_y, 5.0);
        assert_eq!(wmdl.sub_models[1].offset_x, -2.0);
        assert_eq!(wmdl.sub_models[1].offset_y, -2.0);

        let wmdl_path = report.output_path;
        let rt_report = wmdl_to_jsonl(&wmdl_path, None).unwrap();
        let manifest =
            parse_composite_jsonl(&fs::read_to_string(&rt_report.output_path).unwrap(), None);

        assert_eq!(manifest.parts.len(), 3);
        assert_eq!(manifest.parts[0].path, "model/1/main/model.json");
        assert_eq!(manifest.parts[0].x, Some(10.0));
        assert_eq!(manifest.parts[0].y, Some(20.0));
        assert_eq!(manifest.parts[0].xscale, Some(1.5));
        assert_eq!(manifest.parts[1].x, Some(12.0));
        assert_eq!(manifest.parts[1].y, Some(25.0));
        assert_eq!(manifest.parts[2].x, Some(8.0));
        assert_eq!(manifest.parts[2].y, Some(18.0));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_wmdl_assigns_line_numbers_matching_raw_text() {
        let dir = unique_tmp_dir("read");
        let wmdl_path = dir.join("demo.wmdl");
        fs::write(
            &wmdl_path,
            "{\"name\":\"demo\",\"modelRelativePath\":\"a/model.json\",\
             \"figureTemplate\":\"\",\"transformTemplate\":\"\",\
             \"subModels\":[{\"modelRelativePath\":\"b/model.json\",\"offsetX\":1,\"offsetY\":2}],\
             \"x\":5,\"y\":6,\"scale\":1}",
        )
        .unwrap();

        let manifest = read_wmdl(wmdl_path.to_str().unwrap()).unwrap();
        assert_eq!(manifest.parts.len(), 2);
        assert_eq!(manifest.parts[0].line_number, Some(1));
        assert_eq!(manifest.parts[1].line_number, Some(2));
        assert_eq!(manifest.parts[0].path, "a/model.json");
        assert_eq!(manifest.parts[1].x, Some(6.0));
        assert_eq!(manifest.parts[1].y, Some(8.0));
        assert!(!manifest.raw_text.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }
}
