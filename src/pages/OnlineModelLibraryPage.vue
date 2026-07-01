<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import PageHeader from "../components/PageHeader.vue";
import SectionCard from "../components/SectionCard.vue";
import {
  fetchBestdoriBuildData,
  fetchBestdoriIndex,
  formatModelLabel,
  searchBestdoriModels,
  type BestdoriIndex,
  type BestdoriModelEntry,
} from "../lib/bestdori";
import { openPreviewWindow } from "../lib/preview-window";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";
import {
  downloadBestdoriModel,
  pickDirectory,
  upsertResourceEntry,
} from "../lib/tauri";

type LibraryStatus = "queued" | "loading" | "ready" | "error";

type LibraryItem = BestdoriModelEntry & {
  fileCount?: number;
  modelPath?: string;
  outputDir?: string;
  status: LibraryStatus;
  error?: string;
};

const index = ref<BestdoriIndex | null>(null);
const query = ref("");
const libraryDir = ref("");
const previewBackground = ref(DEFAULT_PREVIEW_BACKGROUND);
const libraryItems = ref<LibraryItem[]>([]);
const status = ref("正在加载在线索引");
const isLoadingIndex = ref(false);
const activeModelName = ref("");

const resultModels = computed(() =>
  index.value ? searchBestdoriModels(index.value.models, query.value) : [],
);

const libraryByName = computed(() => {
  const map = new Map<string, LibraryItem>();
  libraryItems.value.forEach((item) => map.set(item.modelName, item));
  return map;
});

const readyCount = computed(() =>
  libraryItems.value.filter((item) => item.status === "ready").length,
);

const queuedCount = computed(() =>
  libraryItems.value.filter((item) => item.status === "queued").length,
);

onMounted(async () => {
  await loadIndex();
});

async function loadIndex() {
  isLoadingIndex.value = true;
  try {
    index.value = await fetchBestdoriIndex();
    status.value = `已加载 ${index.value.models.length} 个在线模型`;
  } catch (error) {
    status.value = error instanceof Error ? error.message : "在线索引加载失败";
  } finally {
    isLoadingIndex.value = false;
  }
}

async function chooseLibraryDir() {
  const selected = await pickDirectory(libraryDir.value);
  if (!selected) return;
  libraryDir.value = selected;
}

async function addToLibrary(model: BestdoriModelEntry) {
  if (libraryByName.value.has(model.modelName)) return;
  libraryItems.value = [...libraryItems.value, { ...model, status: "queued" }];
  activeModelName.value = model.modelName;
  status.value = `已加入导入队列：${model.modelName}`;
  await loadFileCount(model.modelName);
}

async function loadFileCount(modelName: string) {
  try {
    const buildData = await fetchBestdoriBuildData(modelName);
    updateItem(modelName, {
      fileCount: 2 + buildData.textures.length + buildData.motions.length + buildData.expressions.length,
    });
  } catch {
    updateItem(modelName, { fileCount: undefined });
  }
}

function removeFromLibrary(modelName: string) {
  libraryItems.value = libraryItems.value.filter((item) => item.modelName !== modelName);
  if (activeModelName.value === modelName) {
    activeModelName.value = libraryItems.value[0]?.modelName ?? "";
  }
  status.value = `已从导入队列移除：${modelName}`;
}

async function downloadItem(modelName: string): Promise<LibraryItem | null> {
  const item = libraryByName.value.get(modelName);
  if (!item) return null;
  if (!libraryDir.value) {
    status.value = "请先选择下载目录";
    return null;
  }
  if (item.status === "ready" && item.modelPath) return item;

  updateItem(modelName, { status: "loading", error: undefined });
  activeModelName.value = modelName;
  status.value = `正在下载：${modelName}`;
  try {
    const report = await downloadBestdoriModel(modelName, libraryDir.value, modelName);
    updateItem(modelName, {
      status: "ready",
      modelPath: report.modelPath,
      outputDir: report.outputDir,
      fileCount: report.fileCount,
    });
    const latest = libraryByName.value.get(modelName) ?? item;
    await upsertResourceEntry({
      id: `bestdori:${modelName}`,
      name: modelName,
      kind: "live2d",
      source: "bestdori",
      modelPath: report.modelPath,
      rootDir: report.outputDir,
      sourceModelName: modelName,
      displayName: displayName(latest),
      description: formatModelLabel(latest),
      characterId: latest.characterId,
      colorCode: latest.colorCode,
      fileCount: report.fileCount,
    });
    status.value = `已写入数据库：${report.outputDir}`;
    return libraryByName.value.get(modelName) ?? null;
  } catch (error) {
    const message = error instanceof Error ? error.message : "下载失败";
    updateItem(modelName, { status: "error", error: message });
    status.value = message;
    return null;
  }
}

async function downloadQueuedItems() {
  for (const item of libraryItems.value) {
    if (item.status !== "ready") {
      await downloadItem(item.modelName);
    }
  }
}

async function previewItem(modelName: string) {
  const item = await downloadItem(modelName);
  if (!item?.modelPath) return;
  await openPreviewWindow({
    mode: "single",
    background: previewBackground.value,
    sourceLabel: item.modelPath,
    singleModelPath: item.modelPath,
  });
  status.value = `已打开预览：${item.modelName}`;
}

function clearLibrary() {
  libraryItems.value = [];
  activeModelName.value = "";
  status.value = "已清空当前导入队列";
}

function updateItem(modelName: string, patch: Partial<LibraryItem>) {
  libraryItems.value = libraryItems.value.map((item) =>
    item.modelName === modelName ? { ...item, ...patch } : item,
  );
}

function displayName(model: BestdoriModelEntry): string {
  return model.characterNames[3] || model.characterNames[1] || model.characterNames[0] || model.modelName;
}

function statusLabel(item: LibraryItem): string {
  if (item.status === "ready") return "已下载";
  if (item.status === "loading") return "下载中";
  if (item.status === "error") return "失败";
  return "待下载";
}
</script>

<template>
  <PageHeader
    title="在线模型库"
    eyebrow="BESTDORI"
    description="搜索在线 Live2D 资源，下载后写入数据库"
  >
    <template #actions>
      <button type="button" :disabled="isLoadingIndex" @click="loadIndex">
        {{ isLoadingIndex ? "加载中" : "刷新索引" }}
      </button>
      <button type="button" class="ghost" :disabled="!libraryItems.length" @click="clearLibrary">
        清空队列
      </button>
      <button type="button" class="primary" :disabled="!libraryItems.length || !libraryDir" @click="downloadQueuedItems">
        下载待处理资源
      </button>
    </template>
  </PageHeader>

  <div class="page-body">
    <div class="online-layout">
      <SectionCard title="在线搜索" eyebrow="SEARCH">
        <div class="form-stack">
          <label>
            关键词
            <input
              v-model="query"
              placeholder="角色、服装或模型名，例如 爱音 / stage / 001_live_default"
            />
          </label>
          <div class="status-strip">
            <span>{{ index?.generatedAt ? `镜像更新时间 ${index.generatedAt}` : "等待索引" }}</span>
            <span>{{ resultModels.length }} results</span>
          </div>

          <div class="online-results">
            <button
              v-for="model in resultModels"
              :key="model.modelName"
              type="button"
              class="online-model-row"
              :class="{ 'online-model-row--selected': libraryByName.has(model.modelName) }"
              @click="addToLibrary(model)"
            >
              <span class="online-model-row__mark" :style="{ background: model.colorCode || 'var(--accent)' }" />
              <span class="online-model-row__body">
                <strong>{{ model.modelName }}</strong>
                <span>{{ displayName(model) }} · {{ formatModelLabel(model) }}</span>
              </span>
              <span class="online-model-row__action">
                {{ libraryByName.has(model.modelName) ? "已加入" : "加入" }}
              </span>
            </button>
            <div v-if="!resultModels.length" class="online-empty">
              输入关键词后列出可加入导入队列的在线模型
            </div>
          </div>
        </div>
      </SectionCard>

      <SectionCard title="导入队列" eyebrow="QUEUE">
        <div class="form-stack">
          <div class="status-strip">
            <span>{{ readyCount }} 已下载 / {{ queuedCount }} 待下载</span>
            <span>{{ libraryItems.length }} resources</span>
          </div>

          <div class="online-selected-list">
            <div
              v-for="item in libraryItems"
              :key="item.modelName"
              class="online-selected-item"
              :class="{ 'online-selected-item--active': item.modelName === activeModelName }"
            >
              <div>
                <strong>{{ item.modelName }}</strong>
                <span>{{ displayName(item) }} · {{ formatModelLabel(item) }}</span>
              </div>
              <small>{{ item.fileCount ? `${item.fileCount} 文件` : statusLabel(item) }}</small>
              <button type="button" :disabled="item.status === 'loading'" @click="downloadItem(item.modelName)">
                {{ item.status === "ready" ? "更新" : "下载" }}
              </button>
              <button type="button" class="primary" :disabled="item.status === 'loading' || !libraryDir" @click="previewItem(item.modelName)">
                预览
              </button>
              <button type="button" class="ghost" @click="removeFromLibrary(item.modelName)">
                移除
              </button>
            </div>
            <div v-if="!libraryItems.length" class="online-empty">
              从左侧搜索结果中点击“加入”，先把资源放入导入队列
            </div>
          </div>
        </div>
      </SectionCard>

      <SectionCard title="导入设置" eyebrow="STORE">
        <div class="form-stack">
          <label>
            下载目录
            <div class="inline-picker">
              <input v-model="libraryDir" placeholder="选择在线资源下载位置" />
              <button type="button" @click="chooseLibraryDir">浏览</button>
            </div>
          </label>
          <BackgroundSwatchField v-model="previewBackground" />

          <div class="status-strip">
            <span>{{ status }}</span>
            <span>{{ libraryDir || "未选择下载目录" }}</span>
          </div>

          <div class="online-layer-list">
            <div
              v-for="item in libraryItems.filter((entry) => entry.status === 'ready')"
              :key="item.modelName"
              class="online-layer"
            >
              <strong>{{ item.modelName }}</strong>
              <span>{{ item.outputDir }}</span>
            </div>
            <div v-if="!readyCount" class="online-empty online-empty--compact">
              下载完成的资源会显示在这里
            </div>
          </div>
        </div>
      </SectionCard>
    </div>
  </div>
</template>
