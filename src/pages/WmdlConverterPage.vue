<script setup lang="ts">
import { ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
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
  if (!jsonlPath.value) {
    return;
  }
  activity.value = "JSONL -> WMDL";
  const result = await jsonlToWmdl(jsonlPath.value);
  resultText.value = JSON.stringify(result, null, 2);
}

async function convertWmdlToJsonl() {
  if (!wmdlPath.value) {
    return;
  }
  activity.value = "WMDL -> JSONL";
  const result = await wmdlToJsonl(wmdlPath.value, figureRootDir.value || undefined);
  resultText.value = JSON.stringify(result, null, 2);
}
</script>

<template>
  <div class="page-grid page-grid--single">
    <SectionCard title="WMDL 转换" eyebrow="CONVERT">
      <div class="form-stack">
        <label>
          JSONL -> WMDL
          <div class="inline-picker">
            <input v-model="jsonlPath" placeholder="选择 .jsonl" />
            <button type="button" @click="pickJsonlPath">浏览</button>
            <button type="button" @click="convertJsonlToWmdl">转换</button>
          </div>
        </label>

        <label>
          WMDL -> JSONL
          <div class="inline-picker">
            <input v-model="wmdlPath" placeholder="选择 .wmdl" />
            <button type="button" @click="pickWmdlPath">浏览</button>
          </div>
          <div class="inline-picker">
            <input v-model="figureRootDir" placeholder="figure 根目录，可空" />
            <button type="button" @click="pickFigureRoot">目录</button>
            <button type="button" @click="convertWmdlToJsonl">转换</button>
          </div>
        </label>

        <CommandResult :title="activity" :result="resultText || '尚未执行转换。'" />
      </div>
    </SectionCard>
  </div>
</template>
