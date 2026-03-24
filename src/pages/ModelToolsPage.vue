<script setup lang="ts">
import { ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
import SectionCard from "../components/SectionCard.vue";
import {
  batchAddAssets,
  cleanModelJson,
  patchMtnParam,
  pickDirectory,
  pickFile,
  removeMtnParam,
  scanModelDirectory,
} from "../lib/tauri";

const scanDir = ref("");
const cleanupPath = ref("");
const batchTarget = ref("");
const batchAssets = ref("");
const batchPrefix = ref("");
const paramDir = ref("");
const paramName = ref("PARAM_IMPORT");
const paramValue = ref("50");
const activity = ref("等待操作");
const commandResult = ref("");

async function selectScanDir() {
  scanDir.value = (await pickDirectory(scanDir.value)) ?? scanDir.value;
}

async function selectCleanupPath() {
  cleanupPath.value =
    (await pickFile([{ name: "JSON", extensions: ["json"] }], cleanupPath.value)) ??
    cleanupPath.value;
}

async function selectBatchTarget() {
  batchTarget.value =
    (await pickFile(
      [{ name: "Model or JSONL", extensions: ["json", "jsonl"] }],
      batchTarget.value,
    )) ?? batchTarget.value;
}

async function selectBatchAssets() {
  batchAssets.value = (await pickDirectory(batchAssets.value)) ?? batchAssets.value;
}

async function selectParamDir() {
  paramDir.value = (await pickDirectory(paramDir.value)) ?? paramDir.value;
}

async function runScan() {
  activity.value = "扫描目录并生成 model.json 结构";
  const result = await scanModelDirectory(scanDir.value);
  commandResult.value = JSON.stringify(result, null, 2);
}

async function runCleanup() {
  activity.value = "清理重复动作和缺失资源";
  const result = await cleanModelJson(cleanupPath.value);
  commandResult.value = JSON.stringify(result, null, 2);
}

async function runBatchAdd() {
  activity.value = "批量导入动作和表情";
  const result = await batchAddAssets({
    targetPath: batchTarget.value,
    assetSource: batchAssets.value,
    prefix: batchPrefix.value,
  });
  commandResult.value = JSON.stringify(result, null, 2);
}

async function runPatch() {
  activity.value = "批量修改 MTN 参数";
  const result = await patchMtnParam({
    dirPath: paramDir.value,
    paramName: paramName.value,
    value: paramValue.value,
  });
  commandResult.value = JSON.stringify(result, null, 2);
}

async function runRemove() {
  activity.value = "批量删除 MTN 参数";
  const result = await removeMtnParam({
    dirPath: paramDir.value,
    paramName: paramName.value,
  });
  commandResult.value = JSON.stringify(result, null, 2);
}
</script>

<template>
  <div class="page-grid page-grid--single">
    <SectionCard title="模型工具" eyebrow="FILE OPS">
      <div class="form-stack">
        <label>
          扫描目录
          <div class="inline-picker">
            <input v-model="scanDir" placeholder="选择 Live2D 资源目录" />
            <button type="button" @click="selectScanDir">浏览</button>
            <button type="button" @click="runScan">扫描</button>
          </div>
        </label>

        <label>
          清理 model.json
          <div class="inline-picker">
            <input v-model="cleanupPath" placeholder="选择 model.json" />
            <button type="button" @click="selectCleanupPath">浏览</button>
            <button type="button" @click="runCleanup">清理</button>
          </div>
        </label>

        <label>
          批量导入动作 / 表情
          <div class="inline-picker">
            <input v-model="batchTarget" placeholder="target model.json 或 jsonl" />
            <button type="button" @click="selectBatchTarget">目标</button>
          </div>
          <div class="inline-picker">
            <input v-model="batchAssets" placeholder="动作 / 表情目录" />
            <button type="button" @click="selectBatchAssets">资源</button>
          </div>
          <div class="inline-picker">
            <input v-model="batchPrefix" placeholder="前缀，可空" />
            <button type="button" @click="runBatchAdd">执行导入</button>
          </div>
        </label>

        <label>
          MTN 参数批处理
          <div class="inline-picker">
            <input v-model="paramDir" placeholder="包含 .mtn 的目录" />
            <button type="button" @click="selectParamDir">浏览</button>
          </div>
          <div class="inline-picker">
            <input v-model="paramName" placeholder="PARAM_IMPORT" />
            <input v-model="paramValue" placeholder="50" />
            <button type="button" @click="runPatch">写入</button>
            <button type="button" class="ghost" @click="runRemove">删除参数</button>
          </div>
        </label>
        <CommandResult :title="activity" :result="commandResult || '尚未执行命令。'" />
      </div>
    </SectionCard>
  </div>
</template>
