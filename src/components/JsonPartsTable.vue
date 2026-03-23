<script setup lang="ts">
import type { CompositePart, CompositePartType } from "../types/app";

const props = defineProps<{
  parts: CompositePart[];
}>();

const emit = defineEmits<{
  update: [parts: CompositePart[]];
}>();

const typeOptions: CompositePartType[] = ["live2d", "image", "gif", "video"];

function updateField(index: number, key: keyof CompositePart, value: string): void {
  const next = props.parts.map((part) => ({ ...part }));
  const target = next[index] as Record<string, unknown>;

  if (key === "index") {
    target[key] = value ? Number(value) : undefined;
  } else if (["x", "y", "xscale", "yscale"].includes(key)) {
    target[key] = value ? Number(value) : undefined;
  } else if (["loop", "muted", "autoplay", "playsinline"].includes(key)) {
    target[key] = value === "true";
  } else {
    target[key] = value || undefined;
  }

  emit("update", next);
}

function addRow(): void {
  emit("update", [
    ...props.parts,
    {
      path: "",
      type: "live2d",
      id: `part${props.parts.length}`,
      index: props.parts.length,
    },
  ]);
}

function removeRow(index: number): void {
  const next = props.parts.filter((_, current) => current !== index);
  emit(
    "update",
    next.map((part, current) => ({ ...part, index: current })),
  );
}

function move(index: number, direction: -1 | 1): void {
  const target = index + direction;
  if (target < 0 || target >= props.parts.length) {
    return;
  }
  const next = props.parts.map((part) => ({ ...part }));
  [next[index], next[target]] = [next[target], next[index]];
  emit(
    "update",
    next.map((part, current) => ({ ...part, index: current })),
  );
}
</script>

<template>
  <div class="parts-table">
    <div class="parts-table__toolbar">
      <button type="button" @click="addRow">添加图层</button>
    </div>

    <table>
      <thead>
        <tr>
          <th>index</th>
          <th>type</th>
          <th>id</th>
          <th>path</th>
          <th>folder</th>
          <th>x / y</th>
          <th>scale</th>
          <th>flags</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(part, index) in parts" :key="`${part.id ?? 'part'}-${index}`">
          <td>
            <input
              :value="part.index ?? index"
              type="number"
              @input="updateField(index, 'index', ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td>
            <select
              :value="part.type ?? 'live2d'"
              @change="updateField(index, 'type', ($event.target as HTMLSelectElement).value)"
            >
              <option v-for="option in typeOptions" :key="option" :value="option">
                {{ option }}
              </option>
            </select>
          </td>
          <td>
            <input
              :value="part.id ?? ''"
              @input="updateField(index, 'id', ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td>
            <input
              :value="part.path"
              @input="updateField(index, 'path', ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td>
            <input
              :value="part.folder ?? ''"
              @input="updateField(index, 'folder', ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td class="parts-table__pair">
            <input
              :value="part.x ?? ''"
              type="number"
              step="0.1"
              @input="updateField(index, 'x', ($event.target as HTMLInputElement).value)"
            />
            <input
              :value="part.y ?? ''"
              type="number"
              step="0.1"
              @input="updateField(index, 'y', ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td class="parts-table__pair">
            <input
              :value="part.xscale ?? ''"
              type="number"
              step="0.1"
              @input="updateField(index, 'xscale', ($event.target as HTMLInputElement).value)"
            />
            <input
              :value="part.yscale ?? ''"
              type="number"
              step="0.1"
              @input="updateField(index, 'yscale', ($event.target as HTMLInputElement).value)"
            />
          </td>
          <td class="parts-table__flags">
            <label>
              <input
                :checked="part.loop ?? false"
                type="checkbox"
                @change="updateField(index, 'loop', String(($event.target as HTMLInputElement).checked))"
              />
              loop
            </label>
            <label>
              <input
                :checked="part.muted ?? false"
                type="checkbox"
                @change="updateField(index, 'muted', String(($event.target as HTMLInputElement).checked))"
              />
              muted
            </label>
          </td>
          <td class="parts-table__actions">
            <button type="button" @click="move(index, -1)">↑</button>
            <button type="button" @click="move(index, 1)">↓</button>
            <button type="button" @click="removeRow(index)">删</button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
