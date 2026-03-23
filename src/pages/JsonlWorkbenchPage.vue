<script setup lang="ts">
import { computed, ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
import JsonPartsTable from "../components/JsonPartsTable.vue";
import PreviewCanvas from "../components/PreviewCanvas.vue";
import SectionCard from "../components/SectionCard.vue";
import type {
  CompositeManifest,
  PreviewCanvasHandle,
  PreviewStateSnapshot,
  ResolvedCompositeManifest,
} from "../types/app";
import {
  optimizeJsonl,
  pickFile,
  pickSavePath,
  readJsonl,
  resolvePreviewAssets,
  writeJsonl,
} from "../lib/tauri";
import { normalizeSelectorList } from "../lib/manifest";

const filePath = ref("");
const manifest = ref<CompositeManifest>({
  rawText: "",
  parts: [],
  summary: { version: 2, motions: [], expressions: [] },
  diagnostics: [],
});
const action = ref("JSONL 工作台就绪");
const result = ref("");
const previewResolution = ref("");
const previewStatus = ref("未启动预览");
const previewBackground = ref("#000000");
const previewManifest = ref<ResolvedCompositeManifest | null>(null);
const previewState = ref<PreviewStateSnapshot>({
  motions: [],
  expressions: [],
});
const selectedMotion = ref("");
const selectedExpression = ref("");
const previewImportValue = ref<number | undefined>(undefined);
const previewCanvas = ref<PreviewCanvasHandle | null>(null);

const motionsText = computed({
  get: () => (manifest.value.summary.motions ?? []).join(", "),
  set: (value: string) => {
    manifest.value.summary.motions = normalizeSelectorList(value);
  },
});

const expressionsText = computed({
  get: () => (manifest.value.summary.expressions ?? []).join(", "),
  set: (value: string) => {
    manifest.value.summary.expressions = normalizeSelectorList(value);
  },
});

async function openJsonl() {
  const selected = await pickFile([{ name: "JSONL", extensions: ["jsonl"] }], filePath.value);
  if (!selected) {
    return;
  }
  filePath.value = selected;
  manifest.value = await readJsonl(selected);
  action.value = "已读取 JSONL";
  result.value = JSON.stringify(manifest.value.diagnostics, null, 2);
}

async function optimizeCurrent() {
  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  action.value = "已规范化 JSONL";
  result.value = optimized.text;
}

async function saveJsonl() {
  const target =
    (await pickSavePath(filePath.value || undefined, [
      { name: "JSONL", extensions: ["jsonl"] },
    ])) ?? filePath.value;
  if (!target) {
    return;
  }
  filePath.value = target;
  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  const report = await writeJsonl(target, optimized);
  action.value = "已写入 JSONL";
  result.value = JSON.stringify(report, null, 2);
}

async function previewJsonl() {
  if (!filePath.value) {
    return;
  }
  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  const resolved = await resolvePreviewAssets(filePath.value, optimized);
  previewManifest.value = resolved;
  previewResolution.value = JSON.stringify(resolved.parts, null, 2);
  previewStatus.value = "预览资源已装载";
  action.value = "已刷新 JSONL 预览";
}

function onPreviewLoaded(snapshot: PreviewStateSnapshot) {
  previewState.value = snapshot;
  previewStatus.value = "预览已就绪";
  selectedMotion.value = snapshot.motions[0] ?? "";
  selectedExpression.value = snapshot.expressions[0] ?? "";
  previewImportValue.value = snapshot.importValue;
}

function onPreviewError(message: string) {
  previewStatus.value = message;
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
  previewCanvas.value?.applyImport(previewImportValue.value);
}

</script>

<template>
  <div class="page-grid page-grid--wide">
    <SectionCard title="JSONL 工作台" eyebrow="COMPOSITE">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="filePath" placeholder="打开或另存为 .jsonl" />
          <button type="button" @click="openJsonl">打开</button>
          <button type="button" @click="optimizeCurrent">规范化</button>
          <button type="button" @click="saveJsonl">保存</button>
          <button type="button" class="ghost" @click="previewJsonl">刷新预览</button>
        </div>

        <div class="summary-grid">
          <label>
            version
            <input
              :value="manifest.summary.version ?? 2"
              type="number"
              @input="manifest.summary.version = Number(($event.target as HTMLInputElement).value)"
            />
          </label>
          <label>
            import
            <input
              :value="manifest.summary.import ?? ''"
              type="number"
              @input="manifest.summary.import = Number(($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="summary-grid__span">
            motions
            <input v-model="motionsText" placeholder="idle01, tap_body" />
          </label>
          <label class="summary-grid__span">
            expressions
            <input v-model="expressionsText" placeholder="default, smile" />
          </label>
        </div>

        <JsonPartsTable :parts="manifest.parts" @update="manifest.parts = $event" />
      </div>
    </SectionCard>

    <SectionCard title="诊断与解析结果" eyebrow="REPORT">
      <CommandResult :title="action" :result="result || '诊断会显示在这里。'" />
      <CommandResult
        title="预览资源映射"
        :result="previewResolution || '尚未解析。'"
        tone="success"
      />
    </SectionCard>

    <SectionCard title="JSONL 预览" eyebrow="CANVAS">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="previewBackground" placeholder="背景色或渐变" />
          <button type="button" class="ghost" @click="previewJsonl">载入</button>
        </div>

        <div class="summary-grid">
          <label>
            motion
            <select v-model="selectedMotion" @change="applyMotion">
              <option value="">无</option>
              <option v-for="motion in previewState.motions" :key="motion" :value="motion">
                {{ motion }}
              </option>
            </select>
          </label>
          <label>
            expression
            <select v-model="selectedExpression" @change="applyExpression">
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
              v-model="previewImportValue"
              type="number"
              placeholder="可空"
            />
          </label>
          <div class="preview-actions">
            <button type="button" @click="applyMotion">应用动作</button>
            <button type="button" @click="applyExpression">应用表情</button>
            <button type="button" class="ghost" @click="applyImport">应用 import</button>
          </div>
        </div>

        <CommandResult title="预览状态" :result="previewStatus" />

        <PreviewCanvas
          ref="previewCanvas"
          :background="previewBackground"
          :composite-manifest="previewManifest"
          @loaded="onPreviewLoaded"
          @error="onPreviewError"
        />
      </div>
    </SectionCard>
  </div>
</template>
