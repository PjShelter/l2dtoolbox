<script setup lang="ts">
import { computed, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import PageHeader from "../components/PageHeader.vue";
import SectionCard from "../components/SectionCard.vue";
import { openPreviewWindow } from "../lib/preview-window";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";
import { inspectSingleModel } from "../lib/runtime/model-inspector";
import { pickFile, readModelJson, writeModelInitState } from "../lib/tauri";
import type { EditableModelState, ModelInitParam, ModelPartOpacity } from "../types/app";

const filePath = ref("");
const modelVersion = ref("");
const partRows = ref<ModelPartOpacity[]>([]);
const paramRows = ref<ModelInitParam[]>([]);
const status = ref("选择 model.json 后开始编辑");
const previewBackground = ref(DEFAULT_PREVIEW_BACKGROUND);

const canEdit = computed(() => modelVersion.value === "cubism2");

async function chooseModel() {
  const selected = await pickFile([{ name: "Model JSON", extensions: ["json"] }], filePath.value);
  if (!selected) return;
  await loadModel(selected);
}

async function loadModel(path: string) {
  filePath.value = path;
  const document = await readModelJson(path);
  modelVersion.value = document.modelVersion;
  if (document.modelVersion !== "cubism2") {
    partRows.value = [];
    paramRows.value = [];
    status.value = "当前编辑器仅支持 Cubism 2 model.json";
    return;
  }
  const inspected = await inspectSingleModel(path);
  partRows.value = buildOpacityRows(inspected.partIds, document.initOpacities);
  paramRows.value = buildParamRows(inspected.params, document.initParams);
  status.value = `已加载 ${path}`;
}

async function previewCurrent() {
  if (!filePath.value || !canEdit.value) return;
  await openPreviewWindow({
    mode: "single",
    background: previewBackground.value,
    sourceLabel: filePath.value,
    singleModelPath: filePath.value,
    singleModelState: currentState(),
  });
  status.value = "已发送到预览窗口";
}

async function saveCurrent() {
  if (!filePath.value || !canEdit.value) return;
  await writeModelInitState(filePath.value, partRows.value, paramRows.value);
  status.value = `已保存 ${filePath.value}`;
}

function currentState(): EditableModelState {
  return { filePath: filePath.value, modelVersion: modelVersion.value, initOpacities: partRows.value, initParams: paramRows.value };
}

function buildOpacityRows(partIds: string[], existing: ModelPartOpacity[]): ModelPartOpacity[] {
  const map = new Map(existing.map((item) => [item.id, item.value]));
  return partIds.map((id) => ({ id, value: clamp(map.get(id) ?? 1) })).sort((l, r) => l.id.localeCompare(r.id));
}

function buildParamRows(inspected: ModelInitParam[], existing: ModelInitParam[]): ModelInitParam[] {
  const map = new Map(existing.map((item) => [item.id, item.value]));
  return inspected.map((item) => ({ ...item, value: map.get(item.id) ?? item.value }));
}

function clamp(value: number) { return Math.max(0, Math.min(1, value)); }

function updateOpacity(index: number, value: string) {
  const numeric = clamp(Number(value || 0));
  partRows.value = partRows.value.map((row, i) => i === index ? { ...row, value: numeric } : row);
}

function updateParam(index: number, value: string) {
  const numeric = Number(value || 0);
  paramRows.value = paramRows.value.map((row, i) => i === index ? { ...row, value: numeric } : row);
}

function handleDrop(event: DragEvent) {
  const dropped = event.dataTransfer?.files?.[0];
  const droppedPath = (dropped as File & { path?: string } | undefined)?.path;
  if (droppedPath?.toLowerCase().endsWith(".json")) void loadModel(droppedPath);
}
</script>

<template>
  <PageHeader
    title="零件编辑器"
    eyebrow="MODEL"
    description="编辑 Cubism 2 model.json 的零件透明度与初始参数，支持拖入文件"
  >
    <template #actions>
      <button type="button" @click="chooseModel">浏览</button>
      <button type="button" class="ghost" :disabled="!canEdit" @click="previewCurrent">预览</button>
      <button type="button" class="primary" :disabled="!canEdit" @click="saveCurrent">保存</button>
    </template>
  </PageHeader>

  <div class="page-body" @dragover.prevent @drop.prevent="handleDrop">
    <SectionCard title="模型文件" eyebrow="FILE" style="margin-bottom:16px">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="filePath" placeholder="选择或拖入 model.json" />
        </div>
        <BackgroundSwatchField v-model="previewBackground" />
        <div class="status-strip">
          <span>{{ status }}</span>
          <span>{{ modelVersion || "未识别" }}</span>
        </div>
        <p v-if="filePath && !canEdit" class="helper-text" style="margin:0;font-size:12.5px">
          当前编辑器只写回 Cubism 2 model.json。预览页仍可单独打开 Cubism 3 / 4。
        </p>
      </div>
    </SectionCard>

    <div class="page-grid">
      <!-- 零件透明度 -->
      <SectionCard title="init_opacities" eyebrow="PARTS">
        <div class="editor-table">
          <table>
            <thead>
              <tr>
                <th>Part ID</th>
                <th>透明度</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, index) in partRows" :key="row.id">
                <td>{{ row.id }}</td>
                <td>
                  <input
                    :value="row.value"
                    :disabled="!canEdit"
                    max="1" min="0" step="0.01" type="number"
                    @input="updateOpacity(index, ($event.target as HTMLInputElement).value)"
                  />
                </td>
              </tr>
              <tr v-if="!partRows.length">
                <td colspan="2" style="text-align:center;color:var(--label-3);padding:20px 0">
                  加载模型后显示
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </SectionCard>

      <!-- 初始参数 -->
      <SectionCard title="init_params" eyebrow="PARAMS">
        <div class="editor-table">
          <table>
            <thead>
              <tr>
                <th>Param ID</th>
                <th>初始值</th>
                <th>默认值</th>
                <th>最小值</th>
                <th>最大值</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, index) in paramRows" :key="row.id">
                <td>{{ row.id }}</td>
                <td>
                  <input
                    :value="row.value"
                    :disabled="!canEdit"
                    step="0.01" type="number"
                    @input="updateParam(index, ($event.target as HTMLInputElement).value)"
                  />
                </td>
                <td>{{ row.defaultValue ?? "" }}</td>
                <td>{{ row.minValue ?? "" }}</td>
                <td>{{ row.maxValue ?? "" }}</td>
              </tr>
              <tr v-if="!paramRows.length">
                <td colspan="5" style="text-align:center;color:var(--label-3);padding:20px 0">
                  加载模型后显示
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </SectionCard>
    </div>
  </div>
</template>
