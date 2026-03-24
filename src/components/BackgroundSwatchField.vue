<script setup lang="ts">
import { PREVIEW_BACKGROUNDS } from "../lib/preview-backgrounds";

defineProps<{
  modelValue: string;
  label?: string;
  disabled?: boolean;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: string];
}>();
</script>

<template>
  <label class="swatch-field">
    <span>{{ label ?? "背景" }}</span>
    <div class="swatch-field__select">
      <i :style="{ background: modelValue }"></i>
      <select
        :disabled="disabled"
        :value="modelValue"
        @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="option in PREVIEW_BACKGROUNDS" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>
  </label>
</template>
