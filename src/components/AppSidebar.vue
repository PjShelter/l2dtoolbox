<script setup lang="ts">
import type { AppModule } from "../types/app";

defineProps<{
  activeModule: AppModule;
}>();

const emit = defineEmits<{
  select: [module: AppModule];
}>();

const items: Array<{ id: AppModule; label: string }> = [
  { id: "model-tools", label: "模型工具" },
  { id: "jsonl-workbench", label: "JSONL 工作台" },
  { id: "preview", label: "预览" },
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
        {{ item.label }}
      </button>
    </nav>
  </aside>
</template>
