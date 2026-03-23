<script setup lang="ts">
defineProps<{
  title: string;
  eyebrow?: string;
}>();
</script>

<template>
  <section class="section-card">
    <header class="section-card__header">
      <p v-if="eyebrow" class="section-card__eyebrow">{{ eyebrow }}</p>
      <h2>{{ title }}</h2>
    </header>
    <div class="section-card__body">
      <slot />
    </div>
  </section>
</template>
