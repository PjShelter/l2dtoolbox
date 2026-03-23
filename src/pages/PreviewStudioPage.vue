<script setup lang="ts">
import { ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
import PreviewCanvas from "../components/PreviewCanvas.vue";
import SectionCard from "../components/SectionCard.vue";
import type {
  PreviewCanvasHandle,
  PreviewStateSnapshot,
  ResolvedCompositeManifest,
} from "../types/app";
import { pickFile, readJsonl, resolvePreviewAssets } from "../lib/tauri";

const mode = ref<"single" | "composite">("single");
const singleModelPath = ref<string | null>(null);
const compositeManifest = ref<ResolvedCompositeManifest | null>(null);
const background = ref("radial-gradient(circle at top, #2e645f, #091514 72%)");
const status = ref("等待加载预览");
const controls = ref<PreviewStateSnapshot>({
  motions: [],
  expressions: [],
});
const selectedMotion = ref("");
const selectedExpression = ref("");
const importValue = ref<number | undefined>(undefined);
const previewCanvas = ref<PreviewCanvasHandle | null>(null);

async function openSingleModel() {
  const selected = await pickFile([{ name: "model.json", extensions: ["json"] }]);
  if (!selected) {
    return;
  }
  mode.value = "single";
  singleModelPath.value = selected;
  compositeManifest.value = null;
  status.value = `准备加载 ${selected}`;
}

async function openComposite() {
  const selected = await pickFile([{ name: "JSONL", extensions: ["jsonl"] }]);
  if (!selected) {
    return;
  }
  const manifest = await readJsonl(selected);
  compositeManifest.value = await resolvePreviewAssets(selected, manifest);
  mode.value = "composite";
  singleModelPath.value = null;
  status.value = `准备加载 ${selected}`;
}

function onLoaded(snapshot: PreviewStateSnapshot) {
  controls.value = snapshot;
  status.value = "预览已就绪";
  selectedMotion.value = snapshot.motions[0] ?? "";
  selectedExpression.value = snapshot.expressions[0] ?? "";
  importValue.value = snapshot.importValue;
}

function onError(message: string) {
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
</script>

<template>
  <div class="page-grid">
    <SectionCard title="预览控制台" eyebrow="RUNTIME">
      <div class="form-stack">
        <div class="inline-picker">
          <button type="button" @click="openSingleModel">打开单模型</button>
          <button type="button" @click="openComposite">打开 JSONL</button>
        </div>

        <label>
          背景
          <input v-model="background" />
        </label>

        <p class="helper-text">
          预览支持单模型和 JSONL 组合模型。当前模式：
          {{ mode === "single" ? "单模型" : "JSONL" }}
        </p>
      </div>
    </SectionCard>

    <SectionCard title="动作 / 表情" eyebrow="SELECTORS">
      <CommandResult title="状态" :result="status" />
      <div class="form-stack">
        <label>
          motion
          <select v-model="selectedMotion" @change="applyMotion">
            <option value="">无</option>
            <option v-for="motion in controls.motions" :key="motion" :value="motion">
              {{ motion }}
            </option>
          </select>
        </label>
        <label>
          expression
          <select v-model="selectedExpression" @change="applyExpression">
            <option value="">无</option>
            <option v-for="expression in controls.expressions" :key="expression" :value="expression">
              {{ expression }}
            </option>
          </select>
        </label>
        <label>
          import
          <input v-model="importValue" type="number" placeholder="可空" />
        </label>
        <div class="preview-actions">
          <button type="button" @click="applyMotion">应用动作</button>
          <button type="button" @click="applyExpression">应用表情</button>
          <button type="button" class="ghost" @click="applyImport">应用 import</button>
        </div>
      </div>
    </SectionCard>

    <SectionCard title="实时预览" eyebrow="CANVAS">
      <PreviewCanvas
        ref="previewCanvas"
        :background="background"
        :single-model-path="singleModelPath"
        :composite-manifest="compositeManifest"
        @loaded="onLoaded"
        @error="onError"
      />
    </SectionCard>
  </div>
</template>
