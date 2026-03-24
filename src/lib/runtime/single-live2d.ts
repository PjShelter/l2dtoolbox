import type * as PIXI from "pixi.js";
import { Live2DModel } from "pixi-live2d-display-webgal";
import { ensureCubismRuntime } from "./cubism";
import { prepareLive2DModelSource } from "./model-source";
import type { EditableModelState, PreviewLayerState } from "../../types/app";

export type SinglePreviewRuntime = {
  model: Live2DModel;
  motions: string[];
  expressions: string[];
  layers: PreviewLayerState[];
  applyMotion: (name: string) => void;
  applyExpression: (name: string) => void;
  applyImport: (value?: number) => void;
  setLayerVisibility: (key: string, visible: boolean) => void;
  destroy: () => void;
};

export async function loadSingleLive2dModel(
  source: string,
  root: PIXI.Container,
  state?: EditableModelState | null,
): Promise<SinglePreviewRuntime> {
  await ensureCubismRuntime();

  const prepared = await prepareLive2DModelSource(source, {
    initOpacities: state?.initOpacities,
    initParams: state?.initParams,
  });
  const model = await Live2DModel.from(prepared.url);
  model.anchor.set(0.5, 0.5);
  model.scale.set(0.28);
  root.addChild(model);

  return {
    model,
    motions: prepared.motions,
    expressions: prepared.expressions,
    layers: [
      {
        key: "single-model",
        label: "单模型",
        visible: true,
        type: "single",
      },
    ],
    applyMotion: (name: string) => {
      if (
        typeof (model as { motion?: (motionName: string) => Promise<boolean> }).motion ===
        "function"
      ) {
        void (model as { motion: (motionName: string) => Promise<boolean> }).motion(name);
      }
    },
    applyExpression: (name: string) => {
      if (
        typeof (model as { expression?: (expressionName: string) => Promise<boolean> }).expression ===
        "function"
      ) {
        void (model as { expression: (expressionName: string) => Promise<boolean> }).expression(
          name,
        );
      }
    },
    applyImport: () => {
      // Single preview does not currently expose PARAM_IMPORT batch control.
    },
    setLayerVisibility: (_key: string, visible: boolean) => {
      model.visible = visible;
    },
    destroy: () => {
      model.destroy();
      prepared.revoke();
    },
  };
}
