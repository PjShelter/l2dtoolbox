<script setup lang="ts">
import type { AppModule } from "../types/app";

defineProps<{
  activeModule: AppModule;
}>();

const emit = defineEmits<{
  select: [module: AppModule];
}>();

const items: Array<{ id: AppModule; label: string; icon: string }> = [
  { id: "model-tools",     label: "模型工具",    icon: "⚙" },
  { id: "part-editor",     label: "零件编辑器",   icon: "◧" },
  { id: "jsonl-generator", label: "生成 JSONL",  icon: "+" },
  { id: "jsonl-editor",    label: "编辑 JSONL",  icon: "✎" },
  { id: "wmdl-converter",  label: "WMDL 转换",   icon: "⇄" },
  { id: "preset-builder",  label: "一键生成拼好模", icon: "◈" },
  { id: "preview",         label: "预览",        icon: "▶" },
];
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar__title">
      <h1>Live2D 工具箱</h1>
    </div>

    <nav class="sidebar__nav">
      <button
        v-for="item in items"
        :key="item.id"
        class="sidebar__item"
        :class="{ 'sidebar__item--active': item.id === activeModule }"
        @click="emit('select', item.id)"
      >
        <span class="sidebar__item-icon">{{ item.icon }}</span>
        {{ item.label }}
      </button>
    </nav>
  </aside>
</template>
