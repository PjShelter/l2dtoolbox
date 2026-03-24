<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import type {
  PreviewCanvasHandle,
  PreviewStateSnapshot,
  ResolvedCompositeManifest,
} from "../types/app";
import {
  attachPanAndZoom,
  createRenderer,
  syncBackground,
} from "../lib/runtime/pixi";
import { loadSingleLive2dModel } from "../lib/runtime/single-live2d";
import { loadCompositePreview } from "../lib/runtime/composite-live2d";

const props = defineProps<{
  background: string;
  singleModelPath?: string | null;
  compositeManifest?: ResolvedCompositeManifest | null;
}>();

const emit = defineEmits<{
  loaded: [snapshot: PreviewStateSnapshot];
  error: [message: string];
}>();

const host = ref<HTMLElement | null>(null);
const zoom = ref(1);
let renderer: ReturnType<typeof createRenderer> | null = null;
let detachPanZoom: (() => void) | null = null;
let disposeRuntime: (() => void) | null = null;
let motionController: ((name: string) => void) | null = null;
let expressionController: ((name: string) => void) | null = null;
let importController: ((value?: number) => void) | null = null;
let layerVisibilityController: ((key: string, visible: boolean) => void) | null = null;

const loadKey = computed(() =>
  JSON.stringify({
    singleModelPath: props.singleModelPath,
    compositeSource: props.compositeManifest?.source,
    compositeParts: props.compositeManifest?.parts.map((part) => part.resolvedPath),
  }),
);

watch(
  () => props.background,
  (value) => {
    if (host.value) {
      syncBackground(host.value, value);
    }
  },
);

watch(
  loadKey,
  () => {
    void bootstrap();
  },
);

onMounted(() => {
  void bootstrap();
});

onBeforeUnmount(() => {
  teardown();
});

async function bootstrap(): Promise<void> {
  if (!host.value) {
    return;
  }

  teardown();
  renderer = createRenderer(host.value, props.background);
  detachPanZoom = attachPanAndZoom(
    renderer.app.view as HTMLCanvasElement,
    renderer.root,
    {
      value: zoom.value,
      onChange: (value) => {
        zoom.value = value;
      },
    },
  );

  try {
    if (props.singleModelPath) {
      const runtime = await loadSingleLive2dModel(
        props.singleModelPath,
        renderer.root,
      );
      disposeRuntime = runtime.destroy;
      motionController = runtime.applyMotion;
      expressionController = runtime.applyExpression;
      importController = runtime.applyImport;
      layerVisibilityController = runtime.setLayerVisibility;
      emit("loaded", {
        motions: runtime.motions,
        expressions: runtime.expressions,
        layers: runtime.layers,
      });
      return;
    }

    if (props.compositeManifest) {
      const runtime = await loadCompositePreview(
        props.compositeManifest,
        renderer.root,
      );
      disposeRuntime = runtime.destroy;
      motionController = runtime.applyMotion;
      expressionController = runtime.applyExpression;
      importController = runtime.applyImport;
      layerVisibilityController = runtime.setLayerVisibility;
      emit("loaded", {
        motions: runtime.result.selectors.motions,
        expressions: runtime.result.selectors.expressions,
        importValue: props.compositeManifest.summary.import,
        layers: runtime.layers,
      });
    }
  } catch (error) {
    emit("error", error instanceof Error ? error.message : String(error));
  }
}

function teardown(): void {
  disposeRuntime?.();
  disposeRuntime = null;
  motionController = null;
  expressionController = null;
  importController = null;
  layerVisibilityController = null;
  detachPanZoom?.();
  detachPanZoom = null;
  renderer?.destroy();
  renderer = null;
}

function resetViewport(): void {
  zoom.value = 1;
  if (renderer) {
    renderer.root.position.set(
      renderer.app.screen.width / 2,
      renderer.app.screen.height / 2,
    );
    renderer.root.scale.set(1);
  }
}

function applyMotion(name: string): void {
  if (name) {
    motionController?.(name);
  }
}

function applyExpression(name: string): void {
  if (name) {
    expressionController?.(name);
  }
}

function applyImport(value?: number): void {
  importController?.(value);
}

function setLayerVisibility(key: string, visible: boolean): void {
  layerVisibilityController?.(key, visible);
}

defineExpose<PreviewCanvasHandle>({
  resetViewport,
  applyMotion,
  applyExpression,
  applyImport,
  setLayerVisibility,
});
</script>

<template>
  <div class="preview-shell">
    <div ref="host" class="preview-host"></div>
    <button class="preview-reset" type="button" @click="resetViewport">
      重置视口
    </button>
  </div>
</template>
