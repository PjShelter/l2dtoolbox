import type { ModelInitParam, ModelPartOpacity } from "../../types/app";
import { normalizeAssetPath, toAssetUrl } from "../tauri";

export type PreparedLive2DSource = {
  url: string;
  motions: string[];
  expressions: string[];
  revoke: () => void;
};

export type ModelSourceOverrides = {
  initOpacities?: ModelPartOpacity[];
  initParams?: ModelInitParam[];
};

export async function prepareLive2DModelSource(
  modelFilePath: string,
  overrides?: ModelSourceOverrides,
): Promise<PreparedLive2DSource> {
  const modelAssetUrl = toAssetUrl(modelFilePath);
  const response = await fetch(modelAssetUrl);
  if (!response.ok) {
    throw new Error(`Failed to read model.json: ${response.status}`);
  }

  const modelJson = (await response.json()) as Record<string, unknown>;
  const rewritten = rewriteModelJsonPaths(modelJson, modelFilePath, overrides);
  const blob = new Blob([JSON.stringify(rewritten)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);

  return {
    url,
    motions: extractMotions(modelJson),
    expressions: extractExpressions(modelJson),
    revoke: () => URL.revokeObjectURL(url),
  };
}

function rewriteModelJsonPaths(
  modelJson: Record<string, unknown>,
  modelFilePath: string,
  overrides?: ModelSourceOverrides,
): Record<string, unknown> {
  const next = structuredClone(modelJson);
  const modelDir = dirname(modelFilePath);

  rewriteCubism2(next, modelDir);
  rewriteCubism4(next, modelDir);
  applyOverrides(next, overrides);

  return next;
}

function applyOverrides(
  modelJson: Record<string, unknown>,
  overrides?: ModelSourceOverrides,
): void {
  if (!overrides) {
    return;
  }

  if (overrides.initParams?.length) {
    modelJson.init_params = overrides.initParams.map(({ id, value }) => ({
      id,
      value,
    }));
  }

  if (overrides.initOpacities?.length) {
    modelJson.init_opacities = overrides.initOpacities.map(({ id, value }) => ({
      id,
      value,
    }));
  }
}

function rewriteCubism2(modelJson: Record<string, unknown>, modelDir: string): void {
  rewriteStringField(modelJson, "model", modelDir);
  rewriteStringField(modelJson, "physics", modelDir);
  rewriteStringField(modelJson, "pose", modelDir);

  const textures = modelJson.textures;
  if (Array.isArray(textures)) {
    modelJson.textures = textures.map((item) =>
      typeof item === "string" ? resolveAsset(modelDir, item) : item,
    );
  }

  const expressions = modelJson.expressions;
  if (Array.isArray(expressions)) {
    for (const expression of expressions) {
      if (isObject(expression)) {
        rewriteStringField(expression, "file", modelDir);
      }
    }
  }

  const motions = modelJson.motions;
  if (isObject(motions)) {
    for (const group of Object.values(motions)) {
      if (!Array.isArray(group)) {
        continue;
      }
      for (const item of group) {
        if (isObject(item)) {
          rewriteStringField(item, "file", modelDir);
          rewriteStringField(item, "sound", modelDir);
        }
      }
    }
  }
}

function rewriteCubism4(modelJson: Record<string, unknown>, modelDir: string): void {
  const fileReferences = modelJson.FileReferences;
  if (!isObject(fileReferences)) {
    return;
  }

  rewriteStringField(fileReferences, "Moc", modelDir);
  rewriteStringField(fileReferences, "Physics", modelDir);
  rewriteStringField(fileReferences, "Pose", modelDir);
  rewriteStringField(fileReferences, "UserData", modelDir);
  rewriteStringField(fileReferences, "DisplayInfo", modelDir);

  const textures = fileReferences.Textures;
  if (Array.isArray(textures)) {
    fileReferences.Textures = textures.map((item) =>
      typeof item === "string" ? resolveAsset(modelDir, item) : item,
    );
  }

  const expressions = fileReferences.Expressions;
  if (Array.isArray(expressions)) {
    for (const expression of expressions) {
      if (isObject(expression)) {
        rewriteStringField(expression, "File", modelDir);
      }
    }
  }

  const motions = fileReferences.Motions;
  if (isObject(motions)) {
    for (const group of Object.values(motions)) {
      if (!Array.isArray(group)) {
        continue;
      }
      for (const item of group) {
        if (isObject(item)) {
          rewriteStringField(item, "File", modelDir);
          rewriteStringField(item, "Sound", modelDir);
        }
      }
    }
  }
}

function rewriteStringField(
  target: Record<string, unknown>,
  key: string,
  modelDir: string,
): void {
  const value = target[key];
  if (typeof value !== "string" || !value.trim()) {
    return;
  }
  target[key] = resolveAsset(modelDir, value);
}

function resolveAsset(modelDir: string, resourcePath: string): string {
  const normalized = normalizeAssetPath(resourcePath).trim();
  if (!normalized) {
    return normalized;
  }
  if (/^(https?|blob|asset):/i.test(normalized)) {
    return normalized;
  }
  if (/^[A-Za-z]:\//.test(normalized)) {
    return toAssetUrl(normalized);
  }

  const relative = normalized.replace(/^\/+/, "").replace(/^\.\//, "");
  const absolutePath = normalizeJoinedPath(`${modelDir}/${relative}`);
  return toAssetUrl(absolutePath);
}

function normalizeJoinedPath(path: string): string {
  const normalized = normalizeAssetPath(path);
  const prefix = normalized.match(/^[A-Za-z]:\//)?.[0] ?? "";
  const body = prefix ? normalized.slice(prefix.length) : normalized;
  const stack: string[] = [];

  for (const segment of body.split("/")) {
    if (!segment || segment === ".") {
      continue;
    }
    if (segment === "..") {
      stack.pop();
      continue;
    }
    stack.push(segment);
  }

  return `${prefix}${stack.join("/")}`;
}

function dirname(filePath: string): string {
  const normalized = normalizeAssetPath(filePath);
  const index = normalized.lastIndexOf("/");
  return index >= 0 ? normalized.slice(0, index) : normalized;
}

function extractMotions(modelJson: Record<string, unknown>): string[] {
  const motions = modelJson.motions;
  if (isObject(motions)) {
    return Object.keys(motions);
  }

  const fileReferences = modelJson.FileReferences;
  if (isObject(fileReferences) && isObject(fileReferences.Motions)) {
    return Object.keys(fileReferences.Motions);
  }

  return [];
}

function extractExpressions(modelJson: Record<string, unknown>): string[] {
  const cubism2 = modelJson.expressions;
  if (Array.isArray(cubism2)) {
    return cubism2
      .map((item) =>
        isObject(item) && typeof item.name === "string" ? item.name : null,
      )
      .filter((item): item is string => Boolean(item));
  }

  const fileReferences = modelJson.FileReferences;
  if (isObject(fileReferences) && Array.isArray(fileReferences.Expressions)) {
    return fileReferences.Expressions
      .map((item) =>
        isObject(item) && typeof item.Name === "string" ? item.Name : null,
      )
      .filter((item): item is string => Boolean(item));
  }

  return [];
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
