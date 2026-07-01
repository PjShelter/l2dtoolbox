<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import PageHeader from "../components/PageHeader.vue";
import SectionCard from "../components/SectionCard.vue";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";
import { openPreviewWindow } from "../lib/preview-window";
import {
  loadResourceDatabase,
  pickFile,
  removeResourceEntry,
  upsertResourceEntry,
} from "../lib/tauri";
import type { ResourceDatabase, ResourceEntry } from "../types/app";

const database = ref<ResourceDatabase>({ entries: [] });
const query = ref("");
const sourceFilter = ref("all");
const previewBackground = ref(DEFAULT_PREVIEW_BACKGROUND);
const status = ref("正在加载资源库");

const sourceOptions = computed(() => {
  const sources = new Set(database.value.entries.map((entry) => entry.source).filter(Boolean));
  return ["all", ...Array.from(sources).sort((left, right) => left.localeCompare(right))];
});

const filteredEntries = computed(() => {
  const normalized = normalizeSearch(query.value);
  return database.value.entries.filter((entry) => {
    if (sourceFilter.value !== "all" && entry.source !== sourceFilter.value) return false;
    if (!normalized) return true;
    return [
      entry.name,
      entry.displayName,
      entry.description,
      entry.sourceModelName,
      entry.modelPath,
      entry.source,
      entry.kind,
    ].some((value) => normalizeSearch(value ?? "").includes(normalized));
  });
});

onMounted(async () => {
  await reloadDatabase();
});

async function reloadDatabase() {
  database.value = await loadResourceDatabase();
  status.value = `已加载 ${database.value.entries.length} 个资源`;
}

async function addLocalModel() {
  const selected = await pickFile([{ name: "Model JSON", extensions: ["json"] }]);
  if (!selected) return;
  const name = modelNameFromPath(selected);
  database.value = await upsertResourceEntry({
    id: `local:${selected}`,
    name,
    kind: "live2d",
    source: "local",
    modelPath: selected,
    rootDir: dirname(selected),
    displayName: name,
    description: "本地 Live2D model.json",
  });
  status.value = `已添加本地资源：${name}`;
}

async function previewEntry(entry: ResourceEntry) {
  if (!entry.modelPath) {
    status.value = "该资源没有可预览的 model.json 路径";
    return;
  }
  await openPreviewWindow({
    mode: "single",
    background: previewBackground.value,
    sourceLabel: entry.modelPath,
    singleModelPath: entry.modelPath,
  });
  status.value = `已打开预览：${entry.name}`;
}

async function removeEntry(entry: ResourceEntry) {
  database.value = await removeResourceEntry(entry.id);
  status.value = `已移除资源：${entry.name}`;
}

function normalizeSearch(value: string): string {
  return value.normalize("NFKC").trim().toLowerCase().replace(/\s+/g, "");
}

function modelNameFromPath(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const parent = normalized.split("/").slice(-2, -1)[0];
  return parent || normalized.split("/").pop() || "model";
}

function dirname(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const index = normalized.lastIndexOf("/");
  return index >= 0 ? normalized.slice(0, index) : "";
}

function sourceLabel(source: string): string {
  if (source === "bestdori") return "Bestdori";
  if (source === "local") return "本地";
  return source || "未知";
}

function formatTime(value?: string): string {
  if (!value) return "未知";
  const numeric = Number(value);
  if (!Number.isFinite(numeric) || numeric <= 0) return value;
  return new Date(numeric * 1000).toLocaleString();
}
</script>

<template>
  <PageHeader
    title="资源库"
    eyebrow="DATABASE"
    description="统一管理在线导入和本地添加的 Live2D 资源"
  >
    <template #actions>
      <button type="button" @click="reloadDatabase">刷新</button>
      <button type="button" class="primary" @click="addLocalModel">添加本地模型</button>
    </template>
  </PageHeader>

  <div class="page-body">
    <div class="database-layout">
      <SectionCard title="筛选" eyebrow="FILTER">
        <div class="form-stack">
          <label>
            搜索
            <input v-model="query" placeholder="名称、来源、路径" />
          </label>
          <label>
            来源
            <select v-model="sourceFilter">
              <option v-for="source in sourceOptions" :key="source" :value="source">
                {{ source === "all" ? "全部来源" : sourceLabel(source) }}
              </option>
            </select>
          </label>
          <BackgroundSwatchField v-model="previewBackground" />
          <div class="status-strip">
            <span>{{ status }}</span>
            <span>{{ filteredEntries.length }} / {{ database.entries.length }}</span>
          </div>
        </div>
      </SectionCard>

      <SectionCard title="资源" eyebrow="ENTRIES">
        <div class="database-list">
          <div v-for="entry in filteredEntries" :key="entry.id" class="database-row">
            <span class="database-row__mark" :style="{ background: entry.colorCode || 'var(--accent)' }" />
            <div class="database-row__main">
              <strong>{{ entry.name }}</strong>
              <span>{{ entry.displayName || entry.description || entry.modelPath }}</span>
              <small>{{ sourceLabel(entry.source) }} · {{ entry.kind }} · {{ formatTime(entry.updatedAt) }}</small>
            </div>
            <div class="database-row__meta">
              <span>{{ entry.fileCount ? `${entry.fileCount} 文件` : "未统计" }}</span>
              <span>{{ entry.sourceModelName || entry.rootDir }}</span>
            </div>
            <button type="button" class="primary" :disabled="!entry.modelPath" @click="previewEntry(entry)">
              预览
            </button>
            <button type="button" class="ghost" @click="removeEntry(entry)">移除</button>
          </div>
          <div v-if="!filteredEntries.length" class="online-empty">
            当前筛选下没有资源
          </div>
        </div>
      </SectionCard>
    </div>
  </div>
</template>
