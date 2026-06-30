<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import PageHeader from "../components/PageHeader.vue";

interface NameEntry {
  import: number;
  name_ja: string;
  name_en: string;
  name_zh: string;
}

interface DeformerEntry {
  heightLevel: number | null;
  heightRank: number | null;
  OriginX: number | null;
  OriginY: number | null;
}

type DeformerMap = Record<string, DeformerEntry>;

const nameData = ref<NameEntry[]>([]);
const deformerData = ref<DeformerMap>({});
const search = ref("");
const sortByHeight = ref(false);

onMounted(async () => {
  const [namesRes, deformerRes] = await Promise.all([
    fetch("/name_import.json"),
    fetch("/deformer_import.json"),
  ]);
  nameData.value = await namesRes.json();
  deformerData.value = await deformerRes.json();
});

const rows = computed(() => {
  const kw = search.value.trim().toLowerCase();
  let data = nameData.value.filter((item) => {
    if (!kw) return true;
    return [String(item.import), item.name_ja, item.name_en, item.name_zh].some((v) =>
      v.toLowerCase().includes(kw),
    );
  });
  if (sortByHeight.value) {
    data = [...data].sort((a, b) => {
      const ra = deformerData.value[String(a.import)]?.heightRank ?? Infinity;
      const rb = deformerData.value[String(b.import)]?.heightRank ?? Infinity;
      return ra - rb;
    });
  }
  return data.map((item) => {
    const d = deformerData.value[String(item.import)];
    return { ...item, ...d };
  });
});
</script>

<template>
  <PageHeader
    title="IMPORT 参数表"
    eyebrow="REFERENCE"
    description="按 import 值查询角色名称与 Deformer 形变参数"
  >
    <template #actions>
      <button
        type="button"
        :class="sortByHeight ? 'primary' : ''"
        @click="sortByHeight = !sortByHeight"
      >
        {{ sortByHeight ? "身高排序中" : "按身高排序" }}
      </button>
    </template>
  </PageHeader>

  <div class="page-body">
    <div class="import-toolbar">
      <input v-model="search" placeholder="搜索 ID / 日文 / 英文 / 中文…" />
    </div>

    <div class="import-table-wrap">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>日文名</th>
            <th>英文名</th>
            <th>中文名</th>
            <th>体型等级</th>
            <th>身高排名</th>
            <th>OriginX</th>
            <th>OriginY</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="row in rows" :key="row.import">
            <td class="import-table__id">{{ row.import }}</td>
            <td>{{ row.name_ja }}</td>
            <td>{{ row.name_en }}</td>
            <td>{{ row.name_zh }}</td>
            <td class="import-table__num">{{ row.heightLevel ?? "—" }}</td>
            <td class="import-table__num">{{ row.heightRank ?? "—" }}</td>
            <td class="import-table__num">{{ row.OriginX ?? "—" }}</td>
            <td class="import-table__num">{{ row.OriginY ?? "—" }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.import-toolbar {
  margin-bottom: 12px;
}

.import-table-wrap {
  overflow: auto;
  border: 1px solid var(--sep);
  border-radius: var(--r-input);
  max-height: calc(100vh - 180px);
}

table {
  width: 100%;
  border-collapse: collapse;
  background: var(--bg-2);
  font-size: 13px;
}

th, td {
  padding: 6px 10px;
  border-bottom: 1px solid var(--sep);
  text-align: left;
  white-space: nowrap;
}

th {
  position: sticky;
  top: 0;
  background: var(--bg-3);
  font-size: 11.5px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--label-2);
  z-index: 1;
}

.import-table__id  { color: var(--accent); font-weight: 600; width: 56px; }
.import-table__num { text-align: right; color: var(--label-2); width: 80px; }
</style>
