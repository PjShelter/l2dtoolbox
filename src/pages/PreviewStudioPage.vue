<script setup lang="ts">
import { ref } from "vue";
import CommandResult from "../components/CommandResult.vue";
import PreviewCanvas from "../components/PreviewCanvas.vue";
import SectionCard from "../components/SectionCard.vue";
import type { PreviewStateSnapshot, ResolvedCompositeManifest } from "../types/app";
import { pickFile, readJsonl, resolvePreviewAssets } from "../lib/tauri";

const mode = ref<"single" | "composite">("single");
const singleModelPath = ref<string | null>(null);
const compositeManifest = ref<ResolvedCompositeManifest | null>(null);
const background = ref("radial-gradient(circle at top, #2e645f, #091514 72%)");
const status = ref("等待加载预览");
const controls = ref<PreviewStateSnapshot>({
  motions: [],
  expressions: [],
});

async function openSingleModel() {
  const selected = await pickFile([{ name: "model.json", extensions: ["json"] }]);
  if (!selected) {
    return;
  }
  mode.value = "single";
  singleModelPath.value = selected;
  compositeManifest.value = null;
  status.value = `准备加载 ${selected}`;
}

async function openComposite() {
  const selected = await pickFile([{ name: "JSONL", extensions: ["jsonl"] }]);
  if (!selected) {
    return;
  }
  const manifest = await readJsonl(selected);
  compositeManifest.value = await resolvePreviewAssets(selected, manifest);
  mode.value = "composite";
  singleModelPath.value = null;
  status.value = `准备加载 ${selected}`;
}

function onLoaded(snapshot: PreviewStateSnapshot) {
  controls.value = snapshot;
  status.value = "预览已就绪";
}

function onError(message: string) {
  status.value = message;
}
</script>

<template>
  <div class="page-grid">
    <SectionCard title="预览控制台" eyebrow="RUNTIME">
      <div class="form-stack">
        <div class="inline-picker">
          <button type="button" @click="openSingleModel">打开单模型</button>
          <button type="button" @click="openComposite">打开 JSONL</button>
        </div>

        <label>
          背景
          <input v-model="background" />
        </label>

        <p class="helper-text">
          预览支持单模型和 JSONL 组合模型。当前模式：
          {{ mode === "single" ? "单模型" : "JSONL" }}
        </p>
      </div>
    </SectionCard>

    <SectionCard title="动作 / 表情" eyebrow="SELECTORS">
      <CommandResult title="状态" :result="status" />
      <CommandResult
        title="motions"
        :result="controls.motions.join('\n') || '无'"
        tone="success"
      />
      <CommandResult
        title="expressions"
        :result="controls.expressions.join('\n') || '无'"
        tone="success"
      />
    </SectionCard>

    <SectionCard title="实时预览" eyebrow="CANVAS">
      <PreviewCanvas
        :background="background"
        :single-model-path="singleModelPath"
        :composite-manifest="compositeManifest"
        @loaded="onLoaded"
        @error="onError"
      />
    </SectionCard>
  </div>
</template>
