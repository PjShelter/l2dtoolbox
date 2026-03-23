<script setup lang="ts">
import { computed, ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
import JsonPartsTable from "../components/JsonPartsTable.vue";
import SectionCard from "../components/SectionCard.vue";
import type { CompositeManifest } from "../types/app";
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
  const resolved = await resolvePreviewAssets(filePath.value, manifest.value);
  previewResolution.value = JSON.stringify(resolved.parts, null, 2);
  action.value = "已解析预览资源";
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
          <button type="button" class="ghost" @click="previewJsonl">解析预览资源</button>
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
  </div>
</template>
