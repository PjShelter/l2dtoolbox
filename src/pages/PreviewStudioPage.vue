<script setup lang="ts">
import { ref } from "vue";
import BackgroundSwatchField from "../components/BackgroundSwatchField.vue";
import PageHeader from "../components/PageHeader.vue";
import SectionCard from "../components/SectionCard.vue";
import type { ResolvedCompositeManifest } from "../types/app";
import { openPreviewWindow } from "../lib/preview-window";
import { pickFile, readJsonl, readWmdl, resolvePreviewAssets } from "../lib/tauri";
import { DEFAULT_PREVIEW_BACKGROUND } from "../lib/preview-backgrounds";

const singleModelPath = ref<string | null>(null);
const compositeManifest = ref<ResolvedCompositeManifest | null>(null);
const background = ref(DEFAULT_PREVIEW_BACKGROUND);
const status = ref("等待加载预览");

async function openPreviewFile() {
  const selected = await pickFile([
    { name: "预览文件", extensions: ["json", "jsonl", "wmdl"] },
  ]);
  if (!selected) return;

  try {
    const lower = selected.toLowerCase();
    if (lower.endsWith(".wmdl")) {
      const manifest = await readWmdl(selected);
      compositeManifest.value = await resolvePreviewAssets(selected, manifest);
      singleModelPath.value = null;
      await openPreviewWindow({ mode: "composite", background: background.value, sourceLabel: selected, compositeManifest: compositeManifest.value });
    } else if (lower.endsWith(".jsonl")) {
      const manifest = await readJsonl(selected);
      compositeManifest.value = await resolvePreviewAssets(selected, manifest);
      singleModelPath.value = null;
      await openPreviewWindow({ mode: "composite", background: background.value, sourceLabel: selected, compositeManifest: compositeManifest.value });
    } else {
      singleModelPath.value = selected;
      compositeManifest.value = null;
      await openPreviewWindow({ mode: "single", background: background.value, sourceLabel: selected, singleModelPath: selected });
    }
    status.value = `已在子窗口打开 ${selected}`;
  } catch (error) {
    status.value = error instanceof Error ? error.message : String(error);
  }
}

async function reopenCurrent() {
  if (singleModelPath.value) {
    try {
      await openPreviewWindow({ mode: "single", background: background.value, sourceLabel: singleModelPath.value, singleModelPath: singleModelPath.value });
      status.value = `已更新子窗口 ${singleModelPath.value}`;
    } catch (error) {
      status.value = error instanceof Error ? error.message : String(error);
    }
    return;
  }
  if (compositeManifest.value?.source) {
    try {
      await openPreviewWindow({ mode: "composite", background: background.value, sourceLabel: compositeManifest.value.source, compositeManifest: compositeManifest.value });
      status.value = `已更新子窗口 ${compositeManifest.value.source}`;
    } catch (error) {
      status.value = error instanceof Error ? error.message : String(error);
    }
  }
}

const currentFile = () => singleModelPath.value ?? compositeManifest.value?.source ?? null;
</script>

<template>
  <PageHeader
    title="预览"
    eyebrow="RUNTIME"
    description="在独立子窗口中预览 model.json / .jsonl / .wmdl"
  >
    <template #actions>
      <button type="button" @click="openPreviewFile">打开文件</button>
      <button type="button" class="ghost" :disabled="!currentFile()" @click="reopenCurrent">
        重新发送
      </button>
    </template>
  </PageHeader>

  <div class="page-body">
    <SectionCard title="预览控制台" eyebrow="CONTROL">
      <div class="form-stack">
        <BackgroundSwatchField v-model="background" />

        <div class="status-strip">
          <span>{{ status }}</span>
          <span>{{ currentFile() ?? "未选择文件" }}</span>
        </div>
      </div>
    </SectionCard>

    <!-- 空状态提示 -->
    <div
      v-if="!currentFile()"
      style="
        margin-top: 16px;
        min-height: 200px;
        display: grid;
        place-items: center;
        border: 1px dashed var(--sep);
        border-radius: var(--r-card);
        color: var(--label-3);
        text-align: center;
        gap: 6px;
      "
    >
      <div>
        <div style="font-size:28px;margin-bottom:8px;opacity:.4">▶</div>
        <p style="margin:0;font-size:13.5px;color:var(--label-2)">选择预览文件以启动子窗口</p>
        <p style="margin:4px 0 0;font-size:12px;color:var(--label-3)">
          支持 .json（Cubism 2/3/4）、.jsonl、.wmdl
        </p>
      </div>
    </div>

    <!-- 当前文件信息卡 -->
    <div
      v-else
      style="
        margin-top: 16px;
        padding: 14px 18px;
        border: 1px solid var(--sep);
        border-radius: var(--r-card);
        background: var(--bg-2);
        display: grid;
        gap: 4px;
      "
    >
      <p style="margin:0;font-size:11px;font-weight:600;letter-spacing:.06em;text-transform:uppercase;color:var(--accent)">
        当前文件
      </p>
      <p style="margin:0;font-size:13px;color:var(--label);word-break:break-all">
        {{ currentFile() }}
      </p>
      <p style="margin:0;font-size:12px;color:var(--label-2)">
        {{ compositeManifest ? `合成模型 · ${compositeManifest.parts.length} 个层` : '单模型' }}
      </p>
    </div>
  </div>
</template>
