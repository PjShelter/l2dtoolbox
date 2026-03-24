<script setup lang="ts">
import { computed, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import JsonPartsTable from "../components/JsonPartsTable.vue";
import SectionCard from "../components/SectionCard.vue";
import { normalizeSelectorList } from "../lib/manifest";
import { openPreviewWindow } from "../lib/preview-window";
import {
  optimizeJsonl,
  pickFile,
  pickSavePath,
  readJsonl,
  resolvePreviewAssets,
  writeJsonl,
} from "../lib/tauri";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";
import type { CompositeManifest, CompositePart } from "../types/app";

const filePath = ref("");
const manifest = ref<CompositeManifest>({
  rawText: "",
  parts: [],
  summary: { version: 2, motions: [], expressions: [] },
  diagnostics: [],
});
const status = ref("等待打开 JSONL");
const previewBackground = ref(DEFAULT_PREVIEW_BACKGROUND);

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
  await loadFile(selected);
}

async function loadFile(path: string) {
  filePath.value = path;
  manifest.value = await readJsonl(path);
  status.value = `已读取 ${path}`;
}

async function normalizeCurrent() {
  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  status.value = "已规范化当前 JSONL";
}

async function saveCurrent() {
  if (!filePath.value) {
    await saveAsJsonl();
    return;
  }

  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  await writeJsonl(filePath.value, optimized);
  status.value = `已保存 ${filePath.value}`;
}

async function saveAsJsonl() {
  const target = await pickSavePath(filePath.value || undefined, [
    { name: "JSONL", extensions: ["jsonl"] },
  ]);
  if (!target) {
    return;
  }
  filePath.value = target;
  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  await writeJsonl(target, optimized);
  status.value = `已另存为 ${target}`;
}

async function previewJsonl() {
  if (!filePath.value) {
    status.value = "请先打开或保存 JSONL";
    return;
  }

  const optimized = await optimizeJsonl(manifest.value);
  manifest.value = optimized;
  const resolved = await resolvePreviewAssets(filePath.value, optimized);
  await openPreviewWindow({
    mode: "composite",
    background: previewBackground.value,
    sourceLabel: filePath.value,
    compositeManifest: resolved,
  });
  status.value = `已发送到预览窗口：${filePath.value}`;
}

async function pickRowPath(index: number) {
  const basePath = filePath.value ? dirname(filePath.value) : undefined;
  const selected = await pickFile(
    [
      {
        name: "资源",
        extensions: [
          "json",
          "jsonl",
          "model3.json",
          "png",
          "jpg",
          "jpeg",
          "webp",
          "avif",
          "bmp",
          "gif",
          "webm",
          "mp4",
          "ogv",
          "mov",
          "mkv",
        ],
      },
    ],
    basePath,
  );
  if (!selected) {
    return;
  }

  const relativePath = makeRelativePath(selected, basePath);
  const next = manifest.value.parts.map((part) => ({ ...part }));
  next[index] = {
    ...next[index],
    path: relativePath,
    folder: dirname(relativePath),
    type: inferType(relativePath),
  };
  manifest.value.parts = next;
}

function handleDrop(event: DragEvent) {
  const dropped = event.dataTransfer?.files?.[0];
  const droppedPath = (dropped as File & { path?: string } | undefined)?.path;
  if (!dropped) {
    return;
  }
  if (droppedPath?.toLowerCase().endsWith(".jsonl")) {
    void loadFile(droppedPath);
  }
}

function updateParts(parts: CompositePart[]) {
  manifest.value.parts = parts.map((part, index) => ({
    ...part,
    index,
  }));
}

function dirname(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  if (index <= 0) {
    return ".";
  }
  return normalized.slice(0, index);
}

function makeRelativePath(targetPath: string, basePath?: string) {
  if (!basePath) {
    return targetPath.replace(/\\/g, "/");
  }
  const normalizedBasePath = basePath.replace(/\\/g, "/");
  const normalizedTargetPath = targetPath.replace(/\\/g, "/");
  if (/^[A-Za-z]:\//.test(normalizedBasePath) && /^[A-Za-z]:\//.test(normalizedTargetPath)) {
    if (normalizedBasePath.slice(0, 2).toLowerCase() !== normalizedTargetPath.slice(0, 2).toLowerCase()) {
      return normalizedTargetPath;
    }
  }
  const normalizedTarget = normalizedTargetPath.split("/");
  const normalizedBase = normalizedBasePath.split("/");

  while (normalizedTarget.length && normalizedBase.length && normalizedTarget[0] === normalizedBase[0]) {
    normalizedTarget.shift();
    normalizedBase.shift();
  }

  const relative = `${normalizedBase.map(() => "..").join("/")}${normalizedBase.length ? "/" : ""}${normalizedTarget.join("/")}`;
  return relative || ".";
}

function inferType(path: string) {
  const normalized = path.toLowerCase();
  if (normalized.endsWith(".gif")) {
    return "gif";
  }
  if (/\.(webm|mp4|ogv|mov|mkv)$/i.test(normalized)) {
    return "video";
  }
  if (/\.(png|jpg|jpeg|webp|avif|bmp)$/i.test(normalized)) {
    return "image";
  }
  return "live2d";
}
</script>

<template>
  <div class="page-grid page-grid--single page-grid--fill" @dragover.prevent @drop.prevent="handleDrop">
    <SectionCard title="编辑 JSONL" eyebrow="EDITOR">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="filePath" placeholder="拖入或打开 .jsonl" />
          <button type="button" @click="openJsonl">打开</button>
          <button type="button" @click="normalizeCurrent">规范化</button>
          <button type="button" @click="saveCurrent">保存</button>
          <button type="button" @click="saveAsJsonl">另存为</button>
          <button type="button" class="ghost" @click="previewJsonl">预览</button>
        </div>

        <BackgroundSwatchField v-model="previewBackground" />

        <div class="status-strip">
          <span>{{ status }}</span>
          <span>{{ `${manifest.parts.length} 行` }}</span>
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

        <JsonPartsTable :parts="manifest.parts" @update="updateParts" @request-path="pickRowPath" />
      </div>
    </SectionCard>
  </div>
</template>
