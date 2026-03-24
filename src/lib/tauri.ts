import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import type {
  AppSettings,
  BatchAddPayload,
  BatchAddReport,
  CompositeManifest,
  ConversionReport,
  FileWriteReport,
  GeneratedJsonl,
  JsonlGenerationPayload,
  ModelCleanupReport,
  ModelInitParam,
  ModelJsonDocument,
  ModelPartOpacity,
  MtnPatchPayload,
  MtnPatchReport,
  OptimizedCompositeModel,
  PartsPresetMap,
  PresetApplyPayload,
  PresetApplyReport,
  PresetTarget,
  ResolvedCompositeManifest,
  SelectorCopyPayload,
  SelectorCopyReport,
} from "../types/app";

export async function pickDirectory(defaultPath?: string): Promise<string | null> {
  const result = await open({
    directory: true,
    multiple: false,
    defaultPath,
  });
  return typeof result === "string" ? result : null;
}

export async function pickFile(
  filters: { name: string; extensions: string[] }[],
  defaultPath?: string,
): Promise<string | null> {
  const result = await open({
    multiple: false,
    directory: false,
    filters,
    defaultPath,
  });
  return typeof result === "string" ? result : null;
}

export async function pickSavePath(
  defaultPath?: string,
  filters?: { name: string; extensions: string[] }[],
): Promise<string | null> {
  return (await save({ defaultPath, filters })) ?? null;
}

export function toAssetUrl(filePath: string): string {
  return convertFileSrc(normalizeAssetPath(filePath));
}

export function normalizeAssetPath(filePath: string): string {
  return filePath
    .replace(/^\\\\\?\\/, "")
    .replace(/\\/g, "/");
}

export function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("load_settings");
}

export function saveSettings(settings: AppSettings): Promise<AppSettings> {
  return invoke<AppSettings>("save_settings", { settings });
}

export function scanModelDirectory(inputDir: string): Promise<Record<string, unknown>> {
  return invoke("scan_model_directory", { inputDir });
}

export function cleanModelJson(
  filePath: string,
  skipCheck = false,
  autoRemoveMissing = true,
): Promise<ModelCleanupReport> {
  return invoke("clean_model_json", { filePath, skipCheck, autoRemoveMissing });
}

export function batchAddAssets(payload: BatchAddPayload): Promise<BatchAddReport> {
  return invoke("batch_add_assets", { ...payload });
}

export function patchMtnParam(payload: MtnPatchPayload): Promise<MtnPatchReport> {
  return invoke("patch_mtn_param", { ...payload });
}

export function removeMtnParam(payload: MtnPatchPayload): Promise<MtnPatchReport> {
  return invoke("remove_mtn_param", { ...payload });
}

export function readJsonl(filePath: string): Promise<CompositeManifest> {
  return invoke("read_jsonl", { filePath });
}

export function writeJsonl(
  filePath: string,
  manifest: CompositeManifest,
): Promise<FileWriteReport> {
  return invoke("write_jsonl", { filePath, manifest });
}

export function optimizeJsonl(
  manifest: CompositeManifest,
): Promise<OptimizedCompositeModel> {
  return invoke("optimize_jsonl", { manifest });
}

export function resolvePreviewAssets(
  sourcePath: string,
  manifest: CompositeManifest,
): Promise<ResolvedCompositeManifest> {
  return invoke("resolve_preview_assets", { sourcePath, manifest });
}

export function generateJsonlFromSelection(
  payload: JsonlGenerationPayload,
): Promise<GeneratedJsonl> {
  return invoke("generate_jsonl_from_selection", { payload });
}

export function jsonlToWmdl(filePath: string): Promise<ConversionReport> {
  return invoke("jsonl_to_wmdl", { filePath });
}

export function wmdlToJsonl(
  filePath: string,
  figureRootDir?: string,
): Promise<ConversionReport> {
  return invoke("wmdl_to_jsonl", { filePath, figureRootDir });
}

export function readModelJson(filePath: string): Promise<ModelJsonDocument> {
  return invoke("read_model_json", { filePath });
}

export function writeModelInitState(
  filePath: string,
  initOpacities: ModelPartOpacity[],
  initParams: ModelInitParam[],
): Promise<FileWriteReport> {
  return invoke("write_model_init_state", {
    filePath,
    initOpacities,
    initParams,
  });
}

export function readPartsPresets(): Promise<PartsPresetMap> {
  return invoke("read_parts_presets");
}

export function scanPresetTargets(
  rootDir: string,
  maxDepth?: number,
): Promise<PresetTarget[]> {
  return invoke("scan_preset_targets", { rootDir, maxDepth });
}

export function applyOpacityPresets(
  payload: PresetApplyPayload,
): Promise<PresetApplyReport> {
  return invoke("apply_opacity_presets", { payload });
}

export function copyModelSelectors(
  payload: SelectorCopyPayload,
): Promise<SelectorCopyReport> {
  return invoke("copy_model_selectors", { payload });
}
