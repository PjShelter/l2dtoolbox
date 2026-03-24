<script setup lang="ts">
import { listen } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import PreviewCanvas from "../components/PreviewCanvas.vue";
import {
  loadPreviewSession,
  PREVIEW_SESSION_EVENT,
} from "../lib/preview-window";
import type {
  PreviewCanvasHandle,
  PreviewLayerState,
  PreviewSession,
  PreviewStateSnapshot,
} from "../types/app";

const session = ref<PreviewSession | null>(null);
const status = ref("等待预览数据");
const previewState = ref<PreviewStateSnapshot>({
  motions: [],
  expressions: [],
  layers: [],
});
const selectedMotion = ref("");
const selectedExpression = ref("");
const importValue = ref<number | undefined>(undefined);
const layerStates = ref<PreviewLayerState[]>([]);
const previewCanvas = ref<PreviewCanvasHandle | null>(null);
let detachSessionListener: UnlistenFn | null = null;

const modeLabel = computed(() => {
  if (!session.value) {
    return "未载入";
  }
  return session.value.mode === "single" ? "模型 JSON" : "JSONL";
});

function updateBackground(value: string): void {
  if (!session.value) {
    return;
  }
  session.value.background = value;
}

async function applySession(next: PreviewSession | null): Promise<void> {
  session.value = next;
  previewState.value = {
    motions: [],
    expressions: [],
    layers: [],
  };
  selectedMotion.value = "";
  selectedExpression.value = "";
  importValue.value = undefined;
  layerStates.value = [];
  status.value = next ? "加载中" : "等待预览数据";
}

function onPreviewLoaded(snapshot: PreviewStateSnapshot) {
  previewState.value = snapshot;
  selectedMotion.value = snapshot.motions[0] ?? "";
  selectedExpression.value = snapshot.expressions[0] ?? "";
  importValue.value = snapshot.importValue;
  layerStates.value = snapshot.layers.map((layer) => ({ ...layer }));
  status.value = "预览已就绪";
}

function onPreviewError(message: string) {
  status.value = message;
}

function applyMotion() {
  if (selectedMotion.value) {
    previewCanvas.value?.applyMotion(selectedMotion.value);
  }
}

function applyExpression() {
  if (selectedExpression.value) {
    previewCanvas.value?.applyExpression(selectedExpression.value);
  }
}

function applyImport() {
  previewCanvas.value?.applyImport(importValue.value);
}

function toggleLayerVisibility(layerKey: string, visible: boolean) {
  layerStates.value = layerStates.value.map((layer) =>
    layer.key === layerKey ? { ...layer, visible } : layer,
  );
  previewCanvas.value?.setLayerVisibility(layerKey, visible);
}

onMounted(async () => {
  await applySession(loadPreviewSession());
  detachSessionListener = await listen<PreviewSession>(PREVIEW_SESSION_EVENT, (event) => {
    void applySession(event.payload);
  });
});

onBeforeUnmount(() => {
  detachSessionListener?.();
  detachSessionListener = null;
});
</script>

<template>
  <div class="preview-window">
    <div class="preview-window__panel">
      <div class="preview-window__toolbar">
        <div class="preview-window__meta">
          <strong>{{ `已识别 ${modeLabel}` }}</strong>
          <span>{{ session?.sourceLabel ?? "尚未接收到预览任务" }}</span>
        </div>
      </div>

      <div class="preview-window__controls">
        <label class="preview-window__background color-field">
          <span>背景</span>
          <input
            :disabled="!session"
            :value="session?.background ?? '#000000'"
            class="color-picker"
            type="color"
            @input="updateBackground(($event.target as HTMLInputElement).value)"
          />
        </label>

        <label>
          motion
          <select v-model="selectedMotion" :disabled="!previewState.motions.length" @change="applyMotion">
            <option value="">无</option>
            <option v-for="motion in previewState.motions" :key="motion" :value="motion">
              {{ motion }}
            </option>
          </select>
        </label>

        <label>
          expression
          <select
            v-model="selectedExpression"
            :disabled="!previewState.expressions.length"
            @change="applyExpression"
          >
            <option value="">无</option>
            <option
              v-for="expression in previewState.expressions"
              :key="expression"
              :value="expression"
            >
              {{ expression }}
            </option>
          </select>
        </label>

        <label>
          import
          <input
            v-model="importValue"
            type="number"
            placeholder="可空"
            @keydown.enter.prevent="applyImport"
          />
        </label>

        <div class="preview-window__inline-status">
          <span>{{ status }}</span>
          <span v-if="session?.compositeManifest">
            {{ `已解析 ${session.compositeManifest.parts.length} 个图层` }}
          </span>
        </div>
      </div>

      <div v-if="layerStates.length > 1" class="preview-window__layers">
        <span class="preview-window__layers-title">图层显示</span>
        <label
          v-for="layer in layerStates"
          :key="layer.key"
          class="preview-window__layer-toggle"
        >
          <input
            :checked="layer.visible"
            type="checkbox"
            @change="toggleLayerVisibility(layer.key, ($event.target as HTMLInputElement).checked)"
          />
          <span>{{ layer.label }}</span>
          <small>{{ layer.type }}</small>
        </label>
      </div>
    </div>

    <div class="preview-window__canvas">
      <PreviewCanvas
        v-if="session"
        ref="previewCanvas"
        :background="session.background"
        :single-model-path="session.singleModelPath"
        :composite-manifest="session.compositeManifest"
        @loaded="onPreviewLoaded"
        @error="onPreviewError"
      />

      <div v-else class="preview-window__empty">
        点击主窗口中的预览按钮后，这里会显示模型。
      </div>
    </div>
  </div>
</template>
