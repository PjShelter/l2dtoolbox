<script setup lang="ts">
import type { AppModule } from "../types/app";

defineProps<{
  activeModule: AppModule;
}>();

const emit = defineEmits<{
  select: [module: AppModule];
}>();

const groups: Array<{
  label: string;
  items: Array<{ id: AppModule; label: string; icon: string }>;
}> = [
  {
    label: "数据库",
    items: [
      { id: "resource-database", label: "资源库",      icon: "▣" },
      { id: "online-library",    label: "在线模型库",  icon: "◇" },
    ],
  },
  {
    label: "模型",
    items: [
      { id: "model-tools", label: "模型工具",    icon: "⚙" },
      { id: "part-editor", label: "零件编辑器",  icon: "◧" },
    ],
  },
  {
    label: "JSONL",
    items: [
      { id: "jsonl-generator", label: "生成 JSONL", icon: "+" },
      { id: "jsonl-editor",    label: "编辑 JSONL", icon: "✎" },
      { id: "wmdl-converter",  label: "WMDL 转换",  icon: "⇄" },
    ],
  },
  {
    label: "拼好模",
    items: [
      { id: "preset-builder", label: "一键生成拼好模", icon: "◈" },
      { id: "import-table",   label: "IMPORT 参数表",  icon: "⊞" },
    ],
  },
];

const previewItem = { id: "preview" as AppModule, label: "预览", icon: "▶" };
</script>

<template>
  <aside class="sidebar">
    <!-- Logo / Brand -->
    <div class="sidebar__header">
      <div class="sidebar__logo">
        <img class="sidebar__logo-mark" src="/favicon.ico" alt="" aria-hidden="true" />
        <span class="sidebar__logo-text">Live2D 工具箱</span>
      </div>
    </div>

    <!-- Grouped navigation -->
    <div class="sidebar__body">
      <div v-for="group in groups" :key="group.label" class="sidebar__group">
        <p class="sidebar__group-label">{{ group.label }}</p>
        <div class="sidebar__group-items">
          <button
            v-for="item in group.items"
            :key="item.id"
            class="sidebar__item"
            :class="{ 'sidebar__item--active': item.id === activeModule }"
            @click="emit('select', item.id)"
          >
            <span class="sidebar__item-icon">{{ item.icon }}</span>
            {{ item.label }}
          </button>
        </div>
      </div>
    </div>

    <!-- Bottom: preview button -->
    <div class="sidebar__footer">
      <button
        class="sidebar__item"
        :class="{ 'sidebar__item--active': activeModule === previewItem.id }"
        style="width: 100%"
        @click="emit('select', previewItem.id)"
      >
        <span class="sidebar__item-icon">{{ previewItem.icon }}</span>
        {{ previewItem.label }}
      </button>
    </div>
  </aside>
</template>
