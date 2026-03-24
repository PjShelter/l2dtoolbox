<script setup lang="ts">
import { ref } from "vue";
import SectionCard from "../components/SectionCard.vue";
import type { ResolvedCompositeManifest } from "../types/app";
import { openPreviewWindow } from "../lib/preview-window";
import { pickFile, readJsonl, resolvePreviewAssets } from "../lib/tauri";

const mode = ref<"single" | "composite">("single");
const singleModelPath = ref<string | null>(null);
const compositeManifest = ref<ResolvedCompositeManifest | null>(null);
const background = ref("#000000");
const status = ref("等待加载预览");

async function openPreviewFile() {
  const selected = await pickFile([
    { name: "预览文件", extensions: ["json", "jsonl"] },
  ]);
  if (!selected) {
    return;
  }

  try {
    if (selected.toLowerCase().endsWith(".jsonl")) {
      const manifest = await readJsonl(selected);
      compositeManifest.value = await resolvePreviewAssets(selected, manifest);
      mode.value = "composite";
      singleModelPath.value = null;
      await openPreviewWindow({
        mode: "composite",
        background: background.value,
        sourceLabel: selected,
        compositeManifest: compositeManifest.value,
      });
    } else {
      mode.value = "single";
      singleModelPath.value = selected;
      compositeManifest.value = null;
      await openPreviewWindow({
        mode: "single",
        background: background.value,
        sourceLabel: selected,
        singleModelPath: selected,
      });
    }

    status.value = `已在子窗口打开 ${selected}`;
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error);
  }
}

async function reopenCurrent() {
  if (mode.value === "single" && singleModelPath.value) {
    try {
      await openPreviewWindow({
        mode: "single",
        background: background.value,
        sourceLabel: singleModelPath.value,
        singleModelPath: singleModelPath.value,
      });
      status.value = `已更新子窗口 ${singleModelPath.value}`;
    } catch (error) {
      status.value = error instanceof Error ? error.message : String(error);
    }
    return;
  }

  if (mode.value === "composite" && compositeManifest.value?.source) {
    try {
      await openPreviewWindow({
        mode: "composite",
        background: background.value,
        sourceLabel: compositeManifest.value.source,
        compositeManifest: compositeManifest.value,
      });
      status.value = `已更新子窗口 ${compositeManifest.value.source}`;
    } catch (error) {
      status.value = error instanceof Error ? error.message : String(error);
    }
  }
}
</script>

<template>
  <div class="page-grid page-grid--single">
    <SectionCard title="预览控制台" eyebrow="RUNTIME">
      <div class="form-stack">
        <div class="inline-picker">
          <button type="button" @click="openPreviewFile">打开预览文件</button>
          <button type="button" class="ghost" @click="reopenCurrent">重新发送到预览窗口</button>
        </div>

        <label class="color-field">
          背景
          <input v-model="background" class="color-picker" type="color" />
        </label>
      </div>
    </SectionCard>

    <SectionCard title="状态" eyebrow="SESSION">
      <div class="form-stack">
        <div class="status-strip">
          <span>{{ status }}</span>
          <span>{{ mode === "single" ? (singleModelPath ?? "未选择模型") : (compositeManifest?.source ?? "未选择 JSONL") }}</span>
        </div>

        <p class="helper-text">
          动作、表情、import、图层显示和视口控制都在弹出的预览窗口中操作。
        </p>
      </div>
    </SectionCard>
  </div>
</template>
