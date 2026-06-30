<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import AppSidebar from "./components/AppSidebar.vue";
import ModelToolsPage from "./pages/ModelToolsPage.vue";
import PartEditorPage from "./pages/PartEditorPage.vue";
import JsonlGeneratorPage from "./pages/JsonlGeneratorPage.vue";
import JsonlEditorPage from "./pages/JsonlEditorPage.vue";
import ImportTablePage from "./pages/ImportTablePage.vue";
import WmdlConverterPage from "./pages/WmdlConverterPage.vue";
import PresetBuilderPage from "./pages/PresetBuilderPage.vue";
import PreviewStudioPage from "./pages/PreviewStudioPage.vue";
import { loadSettings, saveSettings } from "./lib/tauri";
import type { AppModule, AppSettings } from "./types/app";

const activeModule = ref<AppModule>("model-tools");
const settings = ref<AppSettings | null>(null);
const validModules = new Set<AppModule>([
  "model-tools",
  "part-editor",
  "jsonl-generator",
  "jsonl-editor",
  "wmdl-converter",
  "preset-builder",
  "import-table",
  "preview",
]);

onMounted(async () => {
  settings.value = await loadSettings();
  activeModule.value = validModules.has(settings.value.activeModule)
    ? settings.value.activeModule
    : "model-tools";
});

watch(activeModule, async (value) => {
  if (!settings.value) {
    return;
  }
  settings.value.activeModule = value;
  settings.value = await saveSettings(settings.value);
});
</script>

<template>
  <div class="app-shell">
    <AppSidebar :active-module="activeModule" @select="activeModule = $event" />

    <main class="workspace">
      <ModelToolsPage v-if="activeModule === 'model-tools'" />
      <PartEditorPage v-else-if="activeModule === 'part-editor'" />
      <JsonlGeneratorPage v-else-if="activeModule === 'jsonl-generator'" />
      <JsonlEditorPage v-else-if="activeModule === 'jsonl-editor'" />
      <WmdlConverterPage v-else-if="activeModule === 'wmdl-converter'" />
      <PresetBuilderPage v-else-if="activeModule === 'preset-builder'" />
      <ImportTablePage v-else-if="activeModule === 'import-table'" />
      <PreviewStudioPage v-else />
    </main>
  </div>
</template>
