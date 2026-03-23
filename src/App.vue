<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import AppSidebar from "./components/AppSidebar.vue";
import JsonlWorkbenchPage from "./pages/JsonlWorkbenchPage.vue";
import ModelToolsPage from "./pages/ModelToolsPage.vue";
import PreviewStudioPage from "./pages/PreviewStudioPage.vue";
import { loadSettings, saveSettings } from "./lib/tauri";
import type { AppModule, AppSettings } from "./types/app";

const activeModule = ref<AppModule>("model-tools");
const settings = ref<AppSettings | null>(null);

onMounted(async () => {
  settings.value = await loadSettings();
  activeModule.value = settings.value.activeModule;
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
      <JsonlWorkbenchPage v-else-if="activeModule === 'jsonl-workbench'" />
      <PreviewStudioPage v-else />
    </main>
  </div>
</template>
