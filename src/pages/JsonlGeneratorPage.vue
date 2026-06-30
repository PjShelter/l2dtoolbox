<script setup lang="ts">
import { computed, ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import JsonPartsTable from "../components/JsonPartsTable.vue";
import PageHeader from "../components/PageHeader.vue";
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
  if (!selected) return;
  rootDir.value = selected;
  await listModels();
}

async function listModels() {
  if (!rootDir.value) return;
  const targets = await scanPresetTargets(rootDir.value, 8);
  rows.value = targets.map((target) => ({ path: target.relativePath, checked: true }));
  status.value = `已列出 ${rows.value.length} 个 model.json`;
}

function moveRow(index: number, offset: -1 | 1) {
  const target = index + offset;
  if (target < 0 || target >= rows.value.length) return;
  const next = rows.value.slice();
  [next[index], next[target]] = [next[target], next[index]];
  rows.value = next;
}

async function generate() {
  if (!rootDir.value) { status.value = "请先选择根目录"; return; }
  const selectedRelativePaths = rows.value.filter((r) => r.checked).map((r) => r.path);
  if (!selectedRelativePaths.length) { status.value = "请至少勾选一个模型"; return; }
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
  if (!generated.value || !rootDir.value) return;
  const target = await pickSavePath(`${rootDir.value}\\${suggestedFileName.value}`, [
    { name: "JSONL", extensions: ["jsonl"] },
  ]);
  if (!target) return;
  const optimized = await optimizeJsonl(generated.value);
  generated.value = optimized;
  await writeJsonl(target, optimized);
  status.value = `已保存 ${target}`;
}

async function previewGenerated() {
  if (!generated.value || !rootDir.value) return;
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
  if (!generated.value) return;
  generated.value = { ...generated.value, parts };
}
</script>

<template>
  <PageHeader
    title="生成 JSONL"
    eyebrow="GENERATOR"
    description="从根目录扫描模型列表，按排列顺序生成 .jsonl 合成清单"
  >
    <template #actions>
      <button type="button" :disabled="!rows.length" @click="generate">生成</button>
      <button type="button" class="primary" :disabled="!generated" @click="saveGenerated">
        保存
      </button>
      <button type="button" class="ghost" :disabled="!generated" @click="previewGenerated">
        预览
      </button>
    </template>
  </PageHeader>

  <div class="page-body">
    <div class="page-grid">
      <!-- 左列：来源 + 模型列表 -->
      <SectionCard title="模型源" eyebrow="SOURCE">
        <div class="form-stack">
          <!-- 目录选择 -->
          <div class="inline-picker">
            <input v-model="rootDir" placeholder="选择根目录" />
            <button type="button" @click="chooseRootDir">浏览</button>
            <button type="button" @click="listModels">刷新</button>
          </div>

          <!-- 选项行 -->
          <div class="inline-picker">
            <input v-model="idPrefix" placeholder="ID 前缀" style="max-width:110px" />
            <label class="generator-inline-check">
              <input v-model="useImport" type="checkbox" />
              <span>统一 import</span>
            </label>
            <input
              v-model="importValue"
              :disabled="!useImport"
              placeholder="50"
              style="max-width:64px"
            />
          </div>

          <BackgroundSwatchField v-model="previewBackground" />

          <div class="status-strip">
            <span>{{ status }}</span>
            <span>{{ `已勾选 ${selectedCount} / ${rows.length}` }}</span>
          </div>

          <div class="toolbar">
            <button type="button" @click="toggleAll(true)">全选</button>
            <button type="button" class="ghost" @click="toggleAll(false)">清空</button>
          </div>

          <!-- 模型列表 -->
          <div class="generator-list">
            <table>
              <thead>
                <tr>
                  <th class="generator-list__check">✔</th>
                  <th>路径</th>
                  <th style="width:80px">顺序</th>
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
                <tr v-if="!rows.length">
                  <td colspan="3" style="text-align:center;color:var(--label-3);padding:20px 0">
                    选择根目录后将在此列出模型
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </SectionCard>

      <!-- 右列：生成结果 -->
      <SectionCard title="生成结果" eyebrow="OUTPUT">
        <div class="form-stack">
          <div class="inline-picker">
            <input :value="suggestedFileName" readonly placeholder="尚未生成" />
          </div>

          <template v-if="generated">
            <JsonPartsTable :parts="generated.parts" @update="updateGeneratedParts" />
          </template>
          <div
            v-else
            style="
              min-height:200px;
              display:grid;
              place-items:center;
              border:1px dashed var(--sep);
              border-radius:var(--r-input);
              color:var(--label-3);
              font-size:13px;
            "
          >
            点击「生成」后 JSONL 结构将在此显示
          </div>
        </div>
      </SectionCard>
    </div>
  </div>
</template>
