<script setup lang="ts">
import { computed, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import JsonPartsTable from "../components/JsonPartsTable.vue";
import SectionCard from "../components/SectionCard.vue";
import { openPreviewWindow } from "../lib/preview-window";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";
import {
  generateJsonlFromSelection,
  optimizeJsonl,
  pickDirectory,
  pickSavePath,
  resolvePreviewAssets,
  scanPresetTargets,
  writeJsonl,
} from "../lib/tauri";
import type { CompositeManifest } from "../types/app";

type GeneratorRow = {
  path: string;
  checked: boolean;
};

const rootDir = ref("");
const idPrefix = ref("part");
const useImport = ref(false);
const importValue = ref("50");
const rows = ref<GeneratorRow[]>([]);
const generated = ref<CompositeManifest | null>(null);
const suggestedFileName = ref("model.jsonl");
const status = ref("选择根目录后列出模型");
const previewBackground = ref(DEFAULT_PREVIEW_BACKGROUND);

const selectedCount = computed(() => rows.value.filter((row) => row.checked).length);

async function chooseRootDir() {
  const selected = await pickDirectory(rootDir.value);
  if (!selected) {
    return;
  }
  rootDir.value = selected;
  await listModels();
}

async function listModels() {
  if (!rootDir.value) {
    return;
  }
  const targets = await scanPresetTargets(rootDir.value, 8);
  rows.value = targets.map((target) => ({
    path: target.relativePath,
    checked: true,
  }));
  status.value = `已列出 ${rows.value.length} 个 model.json`;
}

function moveRow(index: number, offset: -1 | 1) {
  const target = index + offset;
  if (target < 0 || target >= rows.value.length) {
    return;
  }
  const next = rows.value.slice();
  [next[index], next[target]] = [next[target], next[index]];
  rows.value = next;
}

async function generate() {
  if (!rootDir.value) {
    status.value = "请先选择根目录";
    return;
  }
  const selectedRelativePaths = rows.value.filter((row) => row.checked).map((row) => row.path);
  if (!selectedRelativePaths.length) {
    status.value = "请至少勾选一个模型";
    return;
  }

  const result = await generateJsonlFromSelection({
    rootDir: rootDir.value,
    selectedRelativePaths,
    idPrefix: idPrefix.value,
    summaryImport: useImport.value ? Number(importValue.value || 0) : undefined,
  });
  generated.value = result.manifest;
  suggestedFileName.value = result.suggestedFileName;
  status.value = `已生成 ${result.selectedCount} 个模型的 JSONL`;
}

async function saveGenerated() {
  if (!generated.value || !rootDir.value) {
    return;
  }
  const target = await pickSavePath(`${rootDir.value}\\${suggestedFileName.value}`, [
    { name: "JSONL", extensions: ["jsonl"] },
  ]);
  if (!target) {
    return;
  }
  const optimized = await optimizeJsonl(generated.value);
  generated.value = optimized;
  await writeJsonl(target, optimized);
  status.value = `已保存 ${target}`;
}

async function previewGenerated() {
  if (!generated.value || !rootDir.value) {
    return;
  }
  const optimized = await optimizeJsonl(generated.value);
  generated.value = optimized;
  const virtualPath = `${rootDir.value}\\${suggestedFileName.value}`;
  const resolved = await resolvePreviewAssets(virtualPath, optimized);
  await openPreviewWindow({
    mode: "composite",
    background: previewBackground.value,
    sourceLabel: virtualPath,
    compositeManifest: resolved,
  });
  status.value = "已发送到预览窗口";
}

function toggleAll(checked: boolean) {
  rows.value = rows.value.map((row) => ({ ...row, checked }));
}

function updateGeneratedParts(parts: CompositeManifest["parts"]) {
  if (!generated.value) {
    return;
  }
  generated.value = {
    ...generated.value,
    parts,
  };
}
</script>

<template>
  <div class="page-grid page-grid--wide">
    <SectionCard title="生成 JSONL" eyebrow="GENERATOR">
      <div class="form-stack">
        <div class="inline-picker">
          <input v-model="rootDir" placeholder="选择根目录" />
          <button type="button" @click="chooseRootDir">浏览</button>
          <button type="button" @click="listModels">列出</button>
        </div>

        <div class="inline-picker">
          <input v-model="idPrefix" placeholder="ID 前缀" />
          <label class="generator-inline-check">
            <input v-model="useImport" type="checkbox" />
            <span>统一 import</span>
          </label>
          <input v-model="importValue" :disabled="!useImport" placeholder="50" />
          <button type="button" @click="generate">生成</button>
        </div>

        <BackgroundSwatchField v-model="previewBackground" />

        <div class="status-strip">
          <span>{{ status }}</span>
          <span>{{ `已勾选 ${selectedCount} / ${rows.length}` }}</span>
        </div>

        <div class="generator-list-actions">
          <button type="button" @click="toggleAll(true)">全选</button>
          <button type="button" class="ghost" @click="toggleAll(false)">清空</button>
        </div>

        <div class="generator-list">
          <table>
            <thead>
              <tr>
                <th>✔</th>
                <th>路径</th>
                <th>顺序</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(row, index) in rows" :key="row.path">
                <td class="generator-list__check">
                  <input v-model="row.checked" type="checkbox" />
                </td>
                <td>{{ row.path }}</td>
                <td class="generator-list__actions">
                  <button type="button" @click="moveRow(index, -1)">↑</button>
                  <button type="button" @click="moveRow(index, 1)">↓</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </SectionCard>

    <SectionCard title="生成结果" eyebrow="OUTPUT">
      <div class="form-stack">
        <div class="inline-picker">
          <input :value="suggestedFileName" readonly />
          <button type="button" :disabled="!generated" @click="saveGenerated">保存</button>
          <button type="button" class="ghost" :disabled="!generated" @click="previewGenerated">
            预览
          </button>
        </div>

        <JsonPartsTable
          v-if="generated"
          :parts="generated.parts"
          @update="updateGeneratedParts"
        />

        <p v-else class="helper-text">生成后会在这里显示 JSONL 结构。</p>
      </div>
    </SectionCard>
  </div>
</template>
