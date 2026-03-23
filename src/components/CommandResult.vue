<script setup lang="ts">
defineProps<{
  title: string;
  result: string;
  tone?: "default" | "success" | "warning";
}>();
</script>

<template>
  <div class="command-result" :data-tone="tone ?? 'default'">
    <p class="command-result__title">{{ title }}</p>
    <pre>{{ result }}</pre>
  </div>
</template>
