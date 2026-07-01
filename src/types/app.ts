export type AppModule =
  | "resource-database"
  | "model-tools"
  | "online-library"
  | "part-editor"
  | "jsonl-generator"
  | "jsonl-editor"
  | "wmdl-converter"
  | "preset-builder"
  | "import-table"
  | "preview";

export type CompositeDiagnosticSeverity = "warning" | "error";
export type CompositePartType = "live2d" | "image" | "gif" | "video";

export interface CompositeDiagnostic {
  code:
    | "invalid-json"
    | "invalid-root"
    | "invalid-summary-field"
    | "invalid-version"
    | "invalid-import"
    | "missing-path"
    | "invalid-path"
    | "invalid-part-type"
    | "invalid-part-flag"
    | "invalid-part-field"
    | "duplicate-summary"
    | "extra-fields"
    | "unknown-line";
  message: string;
  severity: CompositeDiagnosticSeverity;
  lineNumber?: number;
  line?: string;
  field?: string;
}

export interface CompositePart {
  path: string;
  type?: CompositePartType;
  id?: string;
  folder?: string;
  index?: number;
  x?: number;
  y?: number;
  xscale?: number;
  yscale?: number;
  loop?: boolean;
  muted?: boolean;
  autoplay?: boolean;
  playsinline?: boolean;
  lineNumber?: number;
}

export interface CompositeSummary {
  version?: number;
  motions?: string[];
  expressions?: string[];
  import?: number;
  lineNumber?: number;
}

export interface CompositeManifest {
  source?: string;
  rawText: string;
  parts: CompositePart[];
  summary: CompositeSummary;
  diagnostics: CompositeDiagnostic[];
}

export interface OptimizedCompositeModel extends CompositeManifest {
  text: string;
  changed: boolean;
}

export interface ModelCleanupReport {
  filePath: string;
  removedMotions: string[];
  removedExpressions: string[];
  missingFiles: string[];
  changed: boolean;
}

export interface BatchAddPayload {
  targetPath: string;
  assetSource: string;
  prefix?: string;
}

export interface BatchAddReport {
  targetPath: string;
  updatedFiles: string[];
  skippedFiles: string[];
  addedCount: number;
}

export interface MtnPatchPayload {
  dirPath: string;
  paramName: string;
  value?: string;
}

export interface MtnPatchReport {
  dirPath: string;
  paramName: string;
  updatedFiles: string[];
  addedFiles: string[];
  removedFiles: string[];
}

export interface AppSettings {
  recentPaths: string[];
  activeModule: AppModule;
  previewBackground: string;
  lastModelDir?: string;
  lastJsonlDir?: string;
  recentMotion?: string;
  recentExpression?: string;
}

export interface FileWriteReport {
  filePath: string;
  writtenBytes: number;
}

export interface BestdoriDownloadReport {
  modelName: string;
  modelPath: string;
  outputDir: string;
  fileCount: number;
  writtenFiles: string[];
}

export interface ResourceDatabase {
  entries: ResourceEntry[];
}

export interface ResourceEntry {
  id: string;
  name: string;
  kind: string;
  source: string;
  modelPath?: string;
  rootDir?: string;
  sourceModelName?: string;
  displayName?: string;
  description?: string;
  characterId?: string;
  colorCode?: string;
  fileCount?: number;
  createdAt?: string;
  updatedAt?: string;
}

export interface ModelPartOpacity {
  id: string;
  value: number;
}

export interface ModelInitParam {
  id: string;
  value: number;
  defaultValue?: number;
  minValue?: number;
  maxValue?: number;
}

export interface ModelJsonDocument {
  filePath: string;
  modelVersion: string;
  initOpacities: ModelPartOpacity[];
  initParams: ModelInitParam[];
  motions: string[];
  expressions: string[];
  data: Record<string, unknown>;
}

export interface EditableModelState {
  filePath: string;
  modelVersion: string;
  initOpacities: ModelPartOpacity[];
  initParams: ModelInitParam[];
}

export interface JsonlGenerationPayload {
  rootDir: string;
  selectedRelativePaths: string[];
  idPrefix: string;
  summaryImport?: number;
}

export interface GeneratedJsonl {
  manifest: CompositeManifest;
  text: string;
  suggestedFileName: string;
  selectedCount: number;
}

export interface ConversionScannedSelectors {
  motions: string[];
  expressions: string[];
}

export interface ConversionReport {
  inputPath: string;
  outputPath: string;
  warnings: string[];
  scannedSelectors: ConversionScannedSelectors;
}

export type PartsPresetMap = Record<string, string[]>;

export interface PresetTarget {
  modelPath: string;
  relativePath: string;
  detectedPreset: string;
}

export interface PresetSourceScope {
  mode: string;
  subdir?: string;
}

export interface PresetApplyRow {
  modelPath: string;
  presetName: string;
  checked: boolean;
  initOpacities?: ModelPartOpacity[];
}

export interface PresetApplyPayload {
  rootDir: string;
  rows: PresetApplyRow[];
  sourceScope: PresetSourceScope;
  fileMoveMode: string;
}

export interface PresetApplyReport {
  updatedModels: string[];
  exportedAssets: string[];
  skippedAssets: string[];
  warnings: string[];
}

export interface SelectorCopyPayload {
  sourceModelPath: string;
  targetModelPaths: string[];
  fields: string[];
  mode: string;
}

export interface SelectorCopyReport {
  updatedModels: string[];
  warnings: string[];
}

export interface ResolvedCompositePart extends CompositePart {
  resolvedPath: string;
}

export interface ResolvedCompositeManifest extends CompositeManifest {
  parts: ResolvedCompositePart[];
}

export interface PreviewStateSnapshot {
  motions: string[];
  expressions: string[];
  importValue?: number;
  layers: PreviewLayerState[];
}

export interface PreviewLayerState {
  key: string;
  label: string;
  visible: boolean;
  type: CompositePartType | "single";
}

export interface PreviewCanvasHandle {
  resetViewport: () => void;
  applyMotion: (name: string) => void;
  applyExpression: (name: string) => void;
  applyImport: (value?: number) => void;
  setLayerVisibility: (key: string, visible: boolean) => void;
}

export interface PreviewSession {
  mode: "single" | "composite";
  background: string;
  sourceLabel: string;
  singleModelPath?: string | null;
  singleModelState?: EditableModelState | null;
  compositeManifest?: ResolvedCompositeManifest | null;
}
