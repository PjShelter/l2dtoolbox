export type AppModule = "model-tools" | "jsonl-workbench" | "preview";

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
}

export interface PreviewCanvasHandle {
  resetViewport: () => void;
  applyMotion: (name: string) => void;
  applyExpression: (name: string) => void;
  applyImport: (value?: number) => void;
}

export interface PreviewSession {
  mode: "single" | "composite";
  background: string;
  sourceLabel: string;
  singleModelPath?: string | null;
  compositeManifest?: ResolvedCompositeManifest | null;
}
