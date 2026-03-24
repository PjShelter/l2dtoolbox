mod jsonl;
mod model;
mod settings;
mod types;

use tauri::AppHandle;
use types::{
    AppSettings, BatchAddReport, CompositeManifest, ConversionReport, FileWriteReport,
    GeneratedJsonl, ModelCleanupReport, ModelJson, ModelJsonDocument, ModelPartOpacity,
    ModelInitParam, MtnPatchReport, OptimizedCompositeModel, PartsPresetMap, PresetApplyPayload,
    PresetApplyReport, PresetTarget, ResolvedCompositeManifest, SelectorCopyPayload,
    SelectorCopyReport, JsonlGenerationPayload,
};

#[tauri::command]
fn load_settings(app: AppHandle) -> Result<AppSettings, String> {
    settings::load_settings(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    settings::save_settings(&app, settings).map_err(|error| error.to_string())
}

#[tauri::command]
fn scan_model_directory(input_dir: String) -> Result<ModelJson, String> {
    model::scan_model_directory(&input_dir).map_err(|error| error.to_string())
}

#[tauri::command]
fn clean_model_json(
    file_path: String,
    skip_check: bool,
    auto_remove_missing: bool,
) -> Result<ModelCleanupReport, String> {
    model::clean_model_json(&file_path, skip_check, auto_remove_missing)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn batch_add_assets(
    target_path: String,
    asset_source: String,
    prefix: Option<String>,
) -> Result<BatchAddReport, String> {
    model::batch_add_assets(&target_path, &asset_source, prefix.as_deref())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn patch_mtn_param(
    dir_path: String,
    param_name: String,
    value: String,
) -> Result<MtnPatchReport, String> {
    model::patch_mtn_param(&dir_path, &param_name, &value).map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_mtn_param(dir_path: String, param_name: String) -> Result<MtnPatchReport, String> {
    model::remove_mtn_param(&dir_path, &param_name).map_err(|error| error.to_string())
}

#[tauri::command]
fn read_jsonl(file_path: String) -> Result<CompositeManifest, String> {
    jsonl::read_jsonl(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn write_jsonl(file_path: String, manifest: CompositeManifest) -> Result<FileWriteReport, String> {
    let written_bytes = jsonl::write_jsonl(&file_path, manifest).map_err(|error| error.to_string())?;
    Ok(FileWriteReport {
        file_path,
        written_bytes,
    })
}

#[tauri::command]
fn optimize_jsonl(manifest: CompositeManifest) -> Result<OptimizedCompositeModel, String> {
    Ok(jsonl::optimize_jsonl(manifest))
}

#[tauri::command]
fn resolve_preview_assets(
    source_path: String,
    manifest: CompositeManifest,
) -> Result<ResolvedCompositeManifest, String> {
    jsonl::resolve_preview_assets(&source_path, manifest).map_err(|error| error.to_string())
}

#[tauri::command]
fn generate_jsonl_from_selection(payload: JsonlGenerationPayload) -> Result<GeneratedJsonl, String> {
    jsonl::generate_jsonl_from_selection(&payload).map_err(|error| error.to_string())
}

#[tauri::command]
fn jsonl_to_wmdl(file_path: String) -> Result<ConversionReport, String> {
    jsonl::jsonl_to_wmdl(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn wmdl_to_jsonl(
    file_path: String,
    figure_root_dir: Option<String>,
) -> Result<ConversionReport, String> {
    jsonl::wmdl_to_jsonl(&file_path, figure_root_dir.as_deref()).map_err(|error| error.to_string())
}

#[tauri::command]
fn read_model_json(file_path: String) -> Result<ModelJsonDocument, String> {
    model::read_model_json(&file_path).map_err(|error| error.to_string())
}

#[tauri::command]
fn write_model_init_state(
    file_path: String,
    init_opacities: Vec<ModelPartOpacity>,
    init_params: Vec<ModelInitParam>,
) -> Result<FileWriteReport, String> {
    let written_bytes = model::write_model_init_state(&file_path, init_opacities, init_params)
        .map_err(|error| error.to_string())?;
    Ok(FileWriteReport {
        file_path,
        written_bytes,
    })
}

#[tauri::command]
fn read_parts_presets() -> Result<PartsPresetMap, String> {
    model::read_parts_presets().map_err(|error| error.to_string())
}

#[tauri::command]
fn scan_preset_targets(
    root_dir: String,
    max_depth: Option<usize>,
) -> Result<Vec<PresetTarget>, String> {
    model::scan_preset_targets(&root_dir, max_depth).map_err(|error| error.to_string())
}

#[tauri::command]
fn apply_opacity_presets(payload: PresetApplyPayload) -> Result<PresetApplyReport, String> {
    model::apply_opacity_presets(payload).map_err(|error| error.to_string())
}

#[tauri::command]
fn copy_model_selectors(payload: SelectorCopyPayload) -> Result<SelectorCopyReport, String> {
    model::copy_model_selectors(payload).map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            scan_model_directory,
            clean_model_json,
            batch_add_assets,
            patch_mtn_param,
            remove_mtn_param,
            read_jsonl,
            write_jsonl,
            optimize_jsonl,
            resolve_preview_assets,
            generate_jsonl_from_selection,
            jsonl_to_wmdl,
            wmdl_to_jsonl,
            read_model_json,
            write_model_init_state,
            read_parts_presets,
            scan_preset_targets,
            apply_opacity_presets,
            copy_model_selectors
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
