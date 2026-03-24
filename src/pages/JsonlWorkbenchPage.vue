<script setup lang="ts">
import { computed, ref } from "vue";
import JsonPartsTable from "../components/JsonPartsTable.vue";
import SectionCard from "../components/SectionCard.vue";
import type { CompositeManifest } from "../types/app";
import { openPreviewWindow } from "../lib/preview-window";
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
const previewStatus = ref("未启动预览");
const previewBackground = ref("#000000");

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

const diagnosticSummary = computed(() => {
  const diagnostics = manifest.value.diagnostics ?? [];
  if (!diagnostics.length) {
    return "无解析告警";
  }
  const errors = diagnostics.filter((item) => item.severity === "error").length;
  const warnings = diagnostics.length - errors;
  return `解析告警 ${diagnostics.length} 项，错误 ${errors}，警告 ${warnings}`;
});

const pageStatus = computed(
  () => `${action.value}，当前 ${manifest.value.parts.length} 行，${diagnosticSummary.value}`,
);

async function openJsonl() {
  const selected = await pickFile([{ name: "JSONL", extensions: ["jsonl"] }], filePath.value);
  if (!selected) {
    return;
  }
  filePath.value = selected;
  manifest.value = await readJsonl(selected);
  action.value = "已读取 JSONL";
}

async function optimizeCurrent() {
  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  action.value = "已规范化 JSONL";
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
  await writeJsonl(target, optimized);
  action.value = "已写入 JSONL";
}

async function previewJsonl() {
  if (!filePath.value) {
    return;
  }
  try {
    const optimized = await optimizeJsonl(manifest.value);
    manifest.value = optimized;
    const resolved = await resolvePreviewAssets(filePath.value, optimized);
    await openPreviewWindow({
      mode: "composite",
      background: previewBackground.value,
      sourceLabel: filePath.value,
      compositeManifest: resolved,
    });
    previewStatus.value = "已发送到预览子窗口";
    action.value = "已刷新 JSONL 预览";
  } catch (error) {
    previewStatus.value = error instanceof Error ? error.message : String(error);
  }
}

</script>

<template>
  <div class="page-grid page-grid--single">
    <SectionCard title="JSONL 工作台" eyebrow="COMPOSITE">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="filePath" placeholder="打开或另存为 .jsonl" />
          <button type="button" @click="openJsonl">打开</button>
          <button type="button" @click="optimizeCurrent">规范化</button>
          <button type="button" @click="saveJsonl">保存</button>
          <input v-model="previewBackground" class="jsonl-preview-background" placeholder="预览背景" />
          <button type="button" class="ghost" @click="previewJsonl">预览</button>
        </div>

        <div class="status-strip">
          <span>{{ pageStatus }}</span>
          <span>{{ previewStatus }}</span>
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

        <details v-if="manifest.diagnostics.length" class="diagnostic-box">
          <summary>{{ diagnosticSummary }}</summary>
          <ul class="diagnostic-box__list">
            <li
              v-for="(diagnostic, index) in manifest.diagnostics"
              :key="`${diagnostic.code}-${diagnostic.lineNumber ?? index}`"
            >
              <strong>{{ diagnostic.severity === 'error' ? '错误' : '警告' }}</strong>
              {{ diagnostic.lineNumber ? `第 ${diagnostic.lineNumber} 行` : '解析阶段' }}
              {{ diagnostic.message }}
            </li>
          </ul>
        </details>

        <JsonPartsTable :parts="manifest.parts" @update="manifest.parts = $event" />
      </div>
    </SectionCard>
  </div>
</template>
