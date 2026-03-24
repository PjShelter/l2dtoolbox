use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompositeDiagnostic {
    pub code: String,
    pub message: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompositePart {
    pub path: String,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xscale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yscale: Option<f64>,
    #[serde(rename = "loop")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_flag: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoplay: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playsinline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompositeSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expressions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompositeManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub raw_text: String,
    pub parts: Vec<CompositePart>,
    pub summary: CompositeSummary,
    pub diagnostics: Vec<CompositeDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptimizedCompositeModel {
    #[serde(flatten)]
    pub manifest: CompositeManifest,
    pub text: String,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCompositePart {
    #[serde(flatten)]
    pub part: CompositePart,
    pub resolved_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedCompositeManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    pub raw_text: String,
    pub parts: Vec<ResolvedCompositePart>,
    pub summary: CompositeSummary,
    pub diagnostics: Vec<CompositeDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCleanupReport {
    pub file_path: String,
    pub removed_motions: Vec<String>,
    pub removed_expressions: Vec<String>,
    pub missing_files: Vec<String>,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAddReport {
    pub target_path: String,
    pub updated_files: Vec<String>,
    pub skipped_files: Vec<String>,
    pub added_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MtnPatchReport {
    pub dir_path: String,
    pub param_name: String,
    pub updated_files: Vec<String>,
    pub added_files: Vec<String>,
    pub removed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWriteReport {
    pub file_path: String,
    pub written_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelPartOpacity {
    pub id: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelInitParam {
    pub id: String,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelJsonDocument {
    pub file_path: String,
    pub model_version: String,
    pub init_opacities: Vec<ModelPartOpacity>,
    pub init_params: Vec<ModelInitParam>,
    pub motions: Vec<String>,
    pub expressions: Vec<String>,
    pub data: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonlGenerationPayload {
    pub root_dir: String,
    pub selected_relative_paths: Vec<String>,
    pub id_prefix: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_import: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedJsonl {
    pub manifest: CompositeManifest,
    pub text: String,
    pub suggested_file_name: String,
    pub selected_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConversionScannedSelectors {
    pub motions: Vec<String>,
    pub expressions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionReport {
    pub input_path: String,
    pub output_path: String,
    pub warnings: Vec<String>,
    pub scanned_selectors: ConversionScannedSelectors,
}

pub type PartsPresetMap = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetTarget {
    pub model_path: String,
    pub relative_path: String,
    pub detected_preset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetSourceScope {
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetApplyRow {
    pub model_path: String,
    pub preset_name: String,
    pub checked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_opacities: Option<Vec<ModelPartOpacity>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetApplyPayload {
    pub root_dir: String,
    pub rows: Vec<PresetApplyRow>,
    pub source_scope: PresetSourceScope,
    pub file_move_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetApplyReport {
    pub updated_models: Vec<String>,
    pub exported_assets: Vec<String>,
    pub skipped_assets: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorCopyPayload {
    pub source_model_path: String,
    pub target_model_paths: Vec<String>,
    pub fields: Vec<String>,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorCopyReport {
    pub updated_models: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub recent_paths: Vec<String>,
    pub active_module: String,
    pub preview_background: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_model_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_jsonl_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_motion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recent_expression: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            recent_paths: Vec::new(),
            active_module: "model-tools".to_string(),
            preview_background: "#000000".to_string(),
            last_model_dir: None,
            last_jsonl_dir: None,
            recent_motion: None,
            recent_expression: None,
        }
    }
}

pub type ModelJson = Value;
