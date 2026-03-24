<script setup lang="ts">
import { computed, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import SectionCard from "../components/SectionCard.vue";
import { openPreviewWindow } from "../lib/preview-window";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";
import { inspectSingleModel } from "../lib/runtime/model-inspector";
import {
  applyOpacityPresets,
  copyModelSelectors,
  pickDirectory,
  pickFile,
  readModelJson,
  readPartsPresets,
  scanPresetTargets,
} from "../lib/tauri";
import type { ModelPartOpacity, PartsPresetMap, PresetApplyRow } from "../types/app";

type PresetTableRow = {
  modelPath: string;
  relativePath: string;
  detectedPreset: string;
  presetName: string;
  checked: boolean;
  initOpacities?: ModelPartOpacity[];
};

const rootDir = ref("");
const presets = ref<PartsPresetMap>({});
const rows = ref<PresetTableRow[]>([]);
const bulkPreset = ref("保持不变");
const sourceScopeMode = ref<"none" | "all" | "subdir">("none");
const sourceSubdir = ref("");
const fileMoveMode = ref<"copy" | "move">("copy");
const sourceModelPath = ref("");
const selectorMode = ref<"merge" | "overwrite">("merge");
const copyMotions = ref(true);
const copyExpressions = ref(true);
const previewBackground = ref(DEFAULT_PREVIEW_BACKGROUND);
const status = ref("选择根目录后开始处理");
const editingPath = ref("");
const detailRows = ref<ModelPartOpacity[]>([]);
const partCache = new Map<string, string[]>();

const presetOptions = computed(() => ["保持不变", "清空(全0)", ...Object.keys(presets.value)]);
const subdirOptions = computed(() => {
  const names = new Set<string>();
  for (const row of rows.value) {
    const first = row.relativePath.split("/")[0];
    if (first) {
      names.add(first);
    }
  }
  return Array.from(names).sort((left, right) => left.localeCompare(right));
});

async function chooseRootDir() {
  const selected = await pickDirectory(rootDir.value);
  if (!selected) {
    return;
  }
  rootDir.value = selected;
  await loadTargets();
}

async function loadTargets() {
  if (!rootDir.value) {
    return;
  }
  presets.value = await readPartsPresets();
  const targets = await scanPresetTargets(rootDir.value, 2);
  rows.value = targets.map((target) => ({
    ...target,
    presetName: target.detectedPreset === "无" ? "保持不变" : target.detectedPreset,
    checked: true,
  }));
  sourceSubdir.value = subdirOptions.value[0] ?? "";
  bulkPreset.value = presetOptions.value[0] ?? "保持不变";
  status.value = `已加载 ${rows.value.length} 个候选模型`;
}

function applyBulkPreset() {
  rows.value = rows.value.map((row) =>
    row.checked ? { ...row, presetName: bulkPreset.value, initOpacities: undefined } : row,
  );
}

async function previewRow(row: PresetTableRow) {
  const initOpacities = await ensureRowInitOpacities(row);
  await openPreviewWindow({
    mode: "single",
    background: previewBackground.value,
    sourceLabel: row.modelPath,
    singleModelPath: row.modelPath,
    singleModelState: {
      filePath: row.modelPath,
      modelVersion: "cubism2",
      initOpacities,
      initParams: [],
    },
  });
  status.value = `已预览 ${row.relativePath}`;
}

async function openDetail(row: PresetTableRow) {
  const initOpacities = await ensureRowInitOpacities(row);
  editingPath.value = row.modelPath;
  detailRows.value = initOpacities.map((item) => ({ ...item }));
  status.value = `正在编辑 ${row.relativePath}`;
}

function applyDetail() {
  if (!editingPath.value) {
    return;
  }
  rows.value = rows.value.map((row) =>
    row.modelPath === editingPath.value
      ? { ...row, initOpacities: detailRows.value.map((item) => ({ ...item })) }
      : row,
  );
  status.value = "已写入当前行的透明度配置";
}

async function applyPresets() {
  const payloadRows: PresetApplyRow[] = [];

  for (const row of rows.value) {
    if (!row.checked) {
      continue;
    }
    payloadRows.push({
      modelPath: row.modelPath,
      presetName: row.presetName,
      checked: row.checked,
      initOpacities: row.presetName === "保持不变" ? row.initOpacities : await ensureRowInitOpacities(row),
    });
  }

  const report = await applyOpacityPresets({
    rootDir: rootDir.value,
    rows: payloadRows,
    sourceScope: {
      mode: sourceScopeMode.value,
      subdir: sourceScopeMode.value === "subdir" ? sourceSubdir.value : undefined,
    },
    fileMoveMode: fileMoveMode.value,
  });
  status.value = `已更新 ${report.updatedModels.length} 个模型，导出 ${report.exportedAssets.length} 个资源`;
}

async function chooseSourceModel() {
  sourceModelPath.value =
    (await pickFile([{ name: "Model JSON", extensions: ["json"] }], sourceModelPath.value)) ??
    sourceModelPath.value;
}

async function copySelectors() {
  const fields = [
    copyMotions.value ? "motions" : "",
    copyExpressions.value ? "expressions" : "",
  ].filter(Boolean);
  if (!fields.length || !sourceModelPath.value) {
    return;
  }
  const report = await copyModelSelectors({
    sourceModelPath: sourceModelPath.value,
    targetModelPaths: rows.value.filter((row) => row.checked).map((row) => row.modelPath),
    fields,
    mode: selectorMode.value,
  });
  status.value = `已复制 selectors 到 ${report.updatedModels.length} 个目标`;
}

async function ensureRowInitOpacities(row: PresetTableRow): Promise<ModelPartOpacity[]> {
  if (row.modelPath === editingPath.value && detailRows.value.length) {
    return detailRows.value.map((item) => ({ ...item }));
  }
  if (row.initOpacities?.length) {
    return row.initOpacities.map((item) => ({ ...item }));
  }

  const partIds = await getPartIds(row.modelPath);
  const initOpacities = await buildPresetOpacities(row.modelPath, row.presetName, partIds);
  rows.value = rows.value.map((item) =>
    item.modelPath === row.modelPath ? { ...item, initOpacities } : item,
  );
  return initOpacities;
}

async function getPartIds(modelPath: string): Promise<string[]> {
  if (partCache.has(modelPath)) {
    return partCache.get(modelPath)!;
  }
  const inspected = await inspectSingleModel(modelPath);
  partCache.set(modelPath, inspected.partIds);
  return inspected.partIds;
}

async function buildPresetOpacities(
  modelPath: string,
  presetName: string,
  partIds: string[],
): Promise<ModelPartOpacity[]> {
  if (presetName === "保持不变") {
    const document = await readModelJson(modelPath);
    const existing = new Map(document.initOpacities.map((item) => [item.id, item.value]));
    return partIds
      .map((id) => ({
        id,
        value: existing.get(id) ?? 1,
      }))
      .sort((left, right) => left.id.localeCompare(right.id));
  }

  if (presetName === "清空(全0)") {
    return partIds.map((id) => ({ id, value: 0 }));
  }

  const visible = new Set(presets.value[presetName] ?? []);
  return partIds.map((id) => ({
    id,
    value: visible.has(id) ? 1 : 0,
  }));
}

function updateDetail(index: number, value: string) {
  const numeric = Math.max(0, Math.min(1, Number(value || 0)));
  detailRows.value = detailRows.value.map((row, rowIndex) =>
    rowIndex === index ? { ...row, value: numeric } : row,
  );
}
</script>

<template>
  <div class="page-grid page-grid--single">
    <SectionCard title="一键生成拼好模" eyebrow="PRESET">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="rootDir" placeholder="选择根目录" />
          <button type="button" @click="chooseRootDir">浏览</button>
          <button type="button" @click="loadTargets">刷新</button>
        </div>

        <div class="preset-toolbar">
          <label>
            来源范围
            <select v-model="sourceScopeMode">
              <option value="none">不处理</option>
              <option value="all">遍历全部子目录</option>
              <option value="subdir">来源子目录</option>
            </select>
          </label>

          <label v-if="sourceScopeMode === 'subdir'">
            子目录
            <select v-model="sourceSubdir">
              <option v-for="option in subdirOptions" :key="option" :value="option">
                {{ option }}
              </option>
            </select>
          </label>

          <label>
            文件处理
            <select v-model="fileMoveMode">
              <option value="copy">仅复制</option>
              <option value="move">移动</option>
            </select>
          </label>

          <label>
            批量设为
            <select v-model="bulkPreset">
              <option v-for="option in presetOptions" :key="option" :value="option">
                {{ option }}
              </option>
            </select>
          </label>

          <button type="button" @click="applyBulkPreset">批量设为</button>
          <button type="button" @click="applyPresets">应用所选预设</button>
        </div>

        <BackgroundSwatchField v-model="previewBackground" />

        <div class="status-strip">
          <span>{{ status }}</span>
          <span>{{ `${rows.filter((row) => row.checked).length} / ${rows.length}` }}</span>
        </div>

        <div class="generator-list">
          <table>
            <thead>
              <tr>
                <th>✔</th>
                <th>model.json 路径</th>
                <th>检测到的预设</th>
                <th>选择预设</th>
                <th>操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rows" :key="row.modelPath">
                <td class="generator-list__check">
                  <input v-model="row.checked" type="checkbox" />
                </td>
                <td>{{ row.relativePath }}</td>
                <td>{{ row.detectedPreset }}</td>
                <td>
                  <select v-model="row.presetName">
                    <option v-for="option in presetOptions" :key="option" :value="option">
                      {{ option }}
                    </option>
                  </select>
                </td>
                <td class="generator-list__actions">
                  <button type="button" @click="previewRow(row)">查看</button>
                  <button type="button" @click="openDetail(row)">详细</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div class="preset-copy-box">
          <div class="inline-picker">
            <input v-model="sourceModelPath" placeholder="源 model.json" />
            <button type="button" @click="chooseSourceModel">选择源</button>
          </div>

          <div class="preset-toolbar">
            <label>
              复制模式
              <select v-model="selectorMode">
                <option value="merge">合并</option>
                <option value="overwrite">覆盖</option>
              </select>
            </label>
            <label class="generator-inline-check">
              <input v-model="copyMotions" type="checkbox" />
              <span>motions</span>
            </label>
            <label class="generator-inline-check">
              <input v-model="copyExpressions" type="checkbox" />
              <span>expressions</span>
            </label>
            <button type="button" @click="copySelectors">复制到勾选目标</button>
          </div>
        </div>
      </div>
    </SectionCard>

    <SectionCard v-if="editingPath" title="详细透明度" eyebrow="DETAIL">
      <div class="form-stack">
        <div class="inline-picker">
          <input :value="editingPath" readonly />
          <button type="button" @click="applyDetail">写回当前行</button>
        </div>

        <div class="editor-table editor-table--compact">
          <table>
            <thead>
              <tr>
                <th>Part ID</th>
                <th>透明度</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, index) in detailRows" :key="row.id">
                <td>{{ row.id }}</td>
                <td>
                  <input
                    :value="row.value"
                    max="1"
                    min="0"
                    step="0.01"
                    type="number"
                    @input="updateDetail(index, ($event.target as HTMLInputElement).value)"
                  />
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </SectionCard>
  </div>
</template>
