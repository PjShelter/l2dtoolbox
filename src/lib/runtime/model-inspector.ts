import { Live2DModel } from "pixi-live2d-display-webgal";
import type { ModelInitParam } from "../../types/app";
import { ensureCubismRuntime } from "./cubism";
import { prepareLive2DModelSource } from "./model-source";

export type InspectedSingleModel = {
  partIds: string[];
  params: ModelInitParam[];
};

const PART_RE = /^PARTS_/i;
const PARAM_RE = /^PARAM_/i;

export async function inspectSingleModel(modelPath: string): Promise<InspectedSingleModel> {
  await ensureCubismRuntime();
  const prepared = await prepareLive2DModelSource(modelPath);

  try {
    const model = await Live2DModel.from(prepared.url);
    try {
      const anyModel = model as unknown as Record<string, unknown>;
      const internalModel = anyModel.internalModel as Record<string, unknown> | undefined;
      const coreModel = internalModel?.coreModel as Record<string, unknown> | undefined;
      const modelContext =
        typeof coreModel?.getModelContext === "function"
          ? (coreModel.getModelContext as () => unknown)()
          : undefined;

      const partIds = collectStringIds([anyModel, internalModel, coreModel, modelContext], PART_RE);
      const params = collectParams([anyModel, internalModel, coreModel, modelContext]);

      return {
        partIds,
        params,
      };
    } finally {
      model.destroy();
    }
  } finally {
    prepared.revoke();
  }
}

function collectStringIds(sources: unknown[], pattern: RegExp): string[] {
  const ids = new Set<string>();
  const visited = new WeakSet<object>();

  for (const source of sources) {
    walkValue(source, visited, (value) => {
      const text = typeof value === "string" ? value : readIdCandidate(value);
      if (text && pattern.test(text)) {
        ids.add(text);
      }
    });
  }

  return Array.from(ids);
}

function collectParams(sources: unknown[]): ModelInitParam[] {
  const params = new Map<string, ModelInitParam>();
  const visited = new WeakSet<object>();

  for (const source of sources) {
    walkValue(source, visited, (value) => {
      if (!value || typeof value !== "object") {
        return;
      }

      const id = readIdCandidate(value);
      if (!id || !PARAM_RE.test(id)) {
        return;
      }

      const defaultValue = readNumberCandidate(value, ["default", "defaultValue"]);
      const minValue = readNumberCandidate(value, ["min", "minValue"]);
      const maxValue = readNumberCandidate(value, ["max", "maxValue"]);
      const currentValue = readNumberCandidate(value, ["value"]) ?? defaultValue;

      params.set(id, {
        id,
        value: currentValue ?? 0,
        defaultValue,
        minValue,
        maxValue,
      });
    });
  }

  return Array.from(params.values()).sort((left, right) => left.id.localeCompare(right.id));
}

function walkValue(
  value: unknown,
  visited: WeakSet<object>,
  onVisit: (value: unknown) => void,
  depth = 0,
): void {
  if (depth > 8) {
    return;
  }

  onVisit(value);

  if (!value || typeof value !== "object") {
    return;
  }

  if (visited.has(value)) {
    return;
  }
  visited.add(value);

  for (const entry of Object.values(value)) {
    walkValue(entry, visited, onVisit, depth + 1);
  }
}

function readIdCandidate(value: unknown): string | undefined {
  if (!value || typeof value !== "object") {
    return undefined;
  }

  const record = value as Record<string, unknown>;
  for (const key of ["id", "ID", "_id", "_$r"]) {
    const candidate = record[key];
    if (typeof candidate === "string" && candidate.trim()) {
      return candidate.trim();
    }
  }

  return undefined;
}

function readNumberCandidate(
  value: unknown,
  keys: string[],
): number | undefined {
  if (!value || typeof value !== "object") {
    return undefined;
  }

  const record = value as Record<string, unknown>;
  for (const key of keys) {
    const candidate = record[key];
    if (typeof candidate === "number" && Number.isFinite(candidate)) {
      return candidate;
    }
  }

  return undefined;
}
