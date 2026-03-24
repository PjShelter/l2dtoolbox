<script setup lang="ts">
import type { CompositePart, CompositePartType } from "../types/app";

const props = defineProps<{
  parts: CompositePart[];
}>();

const emit = defineEmits<{
  update: [parts: CompositePart[]];
  requestPath: [index: number];
}>();

const typeOptions: CompositePartType[] = ["live2d", "image", "gif", "video"];

function updateField(index: number, key: keyof CompositePart, value: string): void {
  const next = props.parts.map((part) => ({ ...part }));
  const target = next[index] as Record<string, unknown>;

  if (key === "index") {
    target[key] = value ? Number(value) : undefined;
  } else if (["x", "y", "xscale", "yscale"].includes(key)) {
    target[key] = value ? Number(value) : undefined;
  } else {
    target[key] = value || undefined;
  }

  emit("update", next);
}

function updateBooleanField(index: number, key: keyof CompositePart, checked: boolean): void {
  const next = props.parts.map((part) => ({ ...part }));
  const target = next[index] as Record<string, unknown>;
  target[key] = checked || undefined;
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

    <div class="parts-table__scroll">
      <table>
        <thead>
          <tr>
            <th>index</th>
            <th>type</th>
            <th>id</th>
            <th>path</th>
            <th>folder</th>
            <th>x</th>
            <th>y</th>
            <th>xscale</th>
            <th>yscale</th>
            <th>loop</th>
            <th>muted</th>
            <th>autoplay</th>
            <th>playsinline</th>
            <th class="parts-table__actions-header">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(part, index) in parts" :key="`${part.id ?? 'part'}-${index}`">
            <td class="parts-table__number-cell">
              <input
                :value="part.index ?? index"
                class="parts-table__input parts-table__input--number"
                type="number"
                @input="updateField(index, 'index', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td>
              <select
                :value="part.type ?? 'live2d'"
                class="parts-table__input"
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
                class="parts-table__input"
                @input="updateField(index, 'id', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td class="parts-table__path-cell">
              <input
                :value="part.path"
                class="parts-table__input"
                @input="updateField(index, 'path', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td>
              <input
                :value="part.folder ?? ''"
                class="parts-table__input"
                @input="updateField(index, 'folder', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td class="parts-table__number-cell">
              <input
                :value="part.x ?? ''"
                class="parts-table__input parts-table__input--number"
                type="number"
                step="0.1"
                @input="updateField(index, 'x', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td class="parts-table__number-cell">
              <input
                :value="part.y ?? ''"
                class="parts-table__input parts-table__input--number"
                type="number"
                step="0.1"
                @input="updateField(index, 'y', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td class="parts-table__number-cell">
              <input
                :value="part.xscale ?? ''"
                class="parts-table__input parts-table__input--number"
                type="number"
                step="0.1"
                @input="updateField(index, 'xscale', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td class="parts-table__number-cell">
              <input
                :value="part.yscale ?? ''"
                class="parts-table__input parts-table__input--number"
                type="number"
                step="0.1"
                @input="updateField(index, 'yscale', ($event.target as HTMLInputElement).value)"
              />
            </td>
            <td class="parts-table__checkbox-cell">
              <input
                :checked="part.loop ?? false"
                type="checkbox"
                @change="updateBooleanField(index, 'loop', ($event.target as HTMLInputElement).checked)"
              />
            </td>
            <td class="parts-table__checkbox-cell">
              <input
                :checked="part.muted ?? false"
                type="checkbox"
                @change="updateBooleanField(index, 'muted', ($event.target as HTMLInputElement).checked)"
              />
            </td>
            <td class="parts-table__checkbox-cell">
              <input
                :checked="part.autoplay ?? false"
                type="checkbox"
                @change="updateBooleanField(index, 'autoplay', ($event.target as HTMLInputElement).checked)"
              />
            </td>
            <td class="parts-table__checkbox-cell">
              <input
                :checked="part.playsinline ?? false"
                type="checkbox"
                @change="updateBooleanField(index, 'playsinline', ($event.target as HTMLInputElement).checked)"
              />
            </td>
            <td class="parts-table__actions">
              <button type="button" @click="emit('requestPath', index)">路径</button>
              <button type="button" @click="move(index, -1)">↑</button>
              <button type="button" @click="move(index, 1)">↓</button>
              <button type="button" @click="removeRow(index)">删</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
