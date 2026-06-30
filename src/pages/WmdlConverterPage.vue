<script setup lang="ts">
import { ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
import PageHeader from "../components/PageHeader.vue";
import SectionCard from "../components/SectionCard.vue";
import { jsonlToWmdl, pickDirectory, pickFile, wmdlToJsonl } from "../lib/tauri";

const jsonlPath = ref("");
const wmdlPath = ref("");
const figureRootDir = ref("");
const activity = ref("等待转换");
const resultText = ref("");

async function pickJsonlPath() {
  jsonlPath.value =
    (await pickFile([{ name: "JSONL", extensions: ["jsonl"] }], jsonlPath.value)) ?? jsonlPath.value;
}

async function pickWmdlPath() {
  wmdlPath.value =
    (await pickFile([{ name: "WMDL", extensions: ["wmdl"] }], wmdlPath.value)) ?? wmdlPath.value;
}

async function pickFigureRoot() {
  figureRootDir.value = (await pickDirectory(figureRootDir.value)) ?? figureRootDir.value;
}

async function convertJsonlToWmdl() {
  if (!jsonlPath.value) return;
  activity.value = "JSONL → WMDL";
  const result = await jsonlToWmdl(jsonlPath.value);
  resultText.value = JSON.stringify(result, null, 2);
}

async function convertWmdlToJsonl() {
  if (!wmdlPath.value) return;
  activity.value = "WMDL → JSONL";
  const result = await wmdlToJsonl(wmdlPath.value, figureRootDir.value || undefined);
  resultText.value = JSON.stringify(result, null, 2);
}
</script>

<template>
  <PageHeader
    title="WMDL 转换"
    eyebrow="CONVERT"
    description="在 .jsonl 与 .wmdl 格式之间双向转换"
  />

  <div class="page-body">
    <div class="page-grid">
      <!-- JSONL → WMDL -->
      <SectionCard title="JSONL → WMDL" eyebrow="EXPORT">
        <div class="form-stack">
          <p class="helper-text" style="margin:0;font-size:12.5px">
            将 .jsonl 合成清单导出为 .wmdl 格式。
          </p>
          <label>
            源文件
            <div class="inline-picker">
              <input v-model="jsonlPath" placeholder="选择 .jsonl" />
              <button type="button" @click="pickJsonlPath">浏览</button>
            </div>
          </label>
          <button type="button" :disabled="!jsonlPath" @click="convertJsonlToWmdl">
            转换
          </button>
        </div>
      </SectionCard>

      <!-- WMDL → JSONL -->
      <SectionCard title="WMDL → JSONL" eyebrow="IMPORT">
        <div class="form-stack">
          <p class="helper-text" style="margin:0;font-size:12.5px">
            将 .wmdl 文件还原为 .jsonl，可选指定 figure 根目录。
          </p>
          <label>
            源文件
            <div class="inline-picker">
              <input v-model="wmdlPath" placeholder="选择 .wmdl" />
              <button type="button" @click="pickWmdlPath">浏览</button>
            </div>
          </label>
          <label>
            figure 根目录（可选）
            <div class="inline-picker">
              <input v-model="figureRootDir" placeholder="留空则使用文件所在目录" />
              <button type="button" @click="pickFigureRoot">浏览</button>
            </div>
          </label>
          <button type="button" :disabled="!wmdlPath" @click="convertWmdlToJsonl">
            转换
          </button>
        </div>
      </SectionCard>
    </div>

    <CommandResult
      style="margin-top:16px"
      :title="activity"
      :result="resultText || '尚未执行转换。'"
    />
  </div>
</template>
