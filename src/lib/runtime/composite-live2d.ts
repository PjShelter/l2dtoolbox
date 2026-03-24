import type * as PIXI from "pixi.js";
import {
  loadPixiCompositeModel,
  type LoadedPixiCompositeModel,
} from "composite-model";
import type {
  PreviewLayerState,
  ResolvedCompositeManifest,
} from "../../types/app";
import { toAssetUrl } from "../tauri";
import { ensureCubismRuntime } from "./cubism";
import { prepareLive2DModelSource } from "./model-source";

export type CompositePreviewRuntime = {
  result: LoadedPixiCompositeModel;
  layers: PreviewLayerState[];
  applyMotion: (name: string) => void;
  applyExpression: (name: string) => void;
  applyImport: (value?: number) => void;
  setLayerVisibility: (key: string, visible: boolean) => void;
  destroy: () => void;
};

export async function loadCompositePreview(
  manifest: ResolvedCompositeManifest,
  root: PIXI.Container,
): Promise<CompositePreviewRuntime> {
  await ensureCubismRuntime();
  const revokers: Array<() => void> = [];
  const stage = new (await import("pixi.js")).Container();
  stage.position.set(0, 80);
  root.addChild(stage);

  const result = await loadPixiCompositeModel({
    source: manifest.source,
    jsonlText: manifest.rawText,
    resolveAssetUrl: async (part) => {
      const resolved = manifest.parts.find(
        (candidate) => candidate.lineNumber === part.lineNumber,
      );
      if (!resolved?.resolvedPath) {
        throw new Error(`Missing resolved path for ${part.path}`);
      }
      if ((resolved.type ?? "live2d") === "live2d") {
        const prepared = await prepareLive2DModelSource(resolved.resolvedPath);
        revokers.push(prepared.revoke);
        return prepared.url;
      }
      return toAssetUrl(resolved.resolvedPath);
    },
    createContainer: () => stage,
    configureModel: async ({ model, part }) => {
      const baseScale = 0.35;
      const xScale = part.xscale ?? 1;
      const yScale = part.yscale ?? 1;

      (
        model as {
          anchor?: { set?: (x: number, y?: number) => void };
          scale: { set: (x: number, y?: number) => void };
          position: { set: (x: number, y?: number) => void };
        }
      ).anchor?.set?.(0.5, 0.5);

      (
        model as {
          scale: { set: (x: number, y?: number) => void };
          position: { set: (x: number, y?: number) => void };
        }
      ).scale.set(baseScale * xScale, baseScale * yScale);

      (
        model as {
          position: { set: (x: number, y?: number) => void };
        }
      ).position.set(part.x ?? 0, part.y ?? 0);
    },
  });

  const layerEntries = result.nodes.map((node, index) => {
    const key = String(node.part.lineNumber ?? index);
    const label =
      node.part.id?.trim() ||
      node.part.path.split("/").filter(Boolean).pop() ||
      `${node.partType}-${index + 1}`;

    return {
      key,
      label,
      type: node.partType,
      visible: node.displayObject.visible,
      displayObject: node.displayObject,
    };
  });

  return {
    result,
    layers: layerEntries.map(({ key, label, type, visible }) => ({
      key,
      label,
      type,
      visible,
    })),
    applyMotion: (name: string) => {
      result.applyMotion(name);
    },
    applyExpression: (name: string) => {
      result.applyExpression(name);
    },
    applyImport: (value?: number) => {
      result.applyImport(value);
    },
    setLayerVisibility: (key: string, visible: boolean) => {
      const layer = layerEntries.find((entry) => entry.key === key);
      if (!layer) {
        return;
      }
      layer.displayObject.visible = visible;
      layer.visible = visible;
    },
    destroy: () => {
      result.destroy();
      if (stage.parent) {
        stage.parent.removeChild(stage);
      }
      stage.destroy({ children: true });
      for (const revoke of revokers) {
        revoke();
      }
    },
  };
}
