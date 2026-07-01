export type BestdoriBundleFile = {
  bundleName: string;
  fileName: string;
};

export type BestdoriBuildData = {
  model: BestdoriBundleFile;
  physics: BestdoriBundleFile;
  textures: BestdoriBundleFile[];
  motions: BestdoriBundleFile[];
  expressions: BestdoriBundleFile[];
};

export type BestdoriCharacter = {
  characterName?: Array<string | null>;
  nickname?: Array<string | null>;
  colorCode?: string;
};

export type BestdoriCostume = {
  characterId?: number;
  assetBundleName?: string;
  description?: Array<string | null>;
};

export type BestdoriModelEntry = {
  modelName: string;
  characterId: string;
  characterNames: string[];
  costumeDescriptions: string[];
  colorCode?: string;
};

export type BestdoriIndex = {
  generatedAt?: string;
  models: BestdoriModelEntry[];
};

const MIRROR_BASE = "https://live2d.shelter.net.cn/mirror";
const API_BASE = `${MIRROR_BASE}/bestdori-api`;
const ASSETS_BASE = `${MIRROR_BASE}/bestdori-assets`;

const noCacheUrl = (url: string) => `${url}${url.includes("?") ? "&" : "?"}_=${Date.now()}`;

export async function fetchBestdoriIndex(): Promise<BestdoriIndex> {
  const [manifest, characters, assets, costumes] = await Promise.all([
    fetchJson<Record<string, unknown>>(`${MIRROR_BASE}/manifest.json`),
    fetchJson<Record<string, BestdoriCharacter>>(`${API_BASE}/characters/all.2.json`),
    fetchJson<Record<string, unknown>>(`${API_BASE}/explorer/jp/assets/_info.json`),
    fetchJson<Record<string, BestdoriCostume>>(`${API_BASE}/costumes/all.5.json`),
  ]);

  const costumeByAsset = new Map<string, BestdoriCostume>();
  for (const costume of Object.values(costumes)) {
    if (costume?.assetBundleName) {
      costumeByAsset.set(costume.assetBundleName, costume);
    }
  }

  const chara = readNestedRecord(assets, ["live2d", "chara"]);
  const modelNames = Object.keys(chara)
    .filter((name) => !name.endsWith("general"))
    .sort((left, right) => left.localeCompare(right));

  return {
    generatedAt: typeof manifest.generatedAt === "string" ? manifest.generatedAt : undefined,
    models: modelNames.map((modelName) => {
      const characterId = String(Number.parseInt(modelName.slice(0, 3), 10));
      const character = characters[characterId] ?? {};
      const costume = costumeByAsset.get(modelName);
      return {
        modelName,
        characterId,
        characterNames: compactStrings(character.characterName),
        costumeDescriptions: compactStrings(costume?.description),
        colorCode: character.colorCode,
      };
    }),
  };
}

export async function fetchBestdoriBuildData(modelName: string): Promise<BestdoriBuildData> {
  const response = await fetch(
    `${ASSETS_BASE}/jp/live2d/chara/${modelName}_rip/buildData.asset`,
  );
  if (!response.ok) {
    throw new Error(`读取 buildData 失败：HTTP ${response.status}`);
  }
  const payload = (await response.json()) as { Base?: BestdoriBuildData };
  if (!payload.Base?.model) {
    throw new Error(`buildData 缺少 Base：${modelName}`);
  }
  return payload.Base;
}

export function searchBestdoriModels(
  models: BestdoriModelEntry[],
  query: string,
): BestdoriModelEntry[] {
  const normalized = normalizeSearchText(query);
  if (!normalized) return [];
  return models
    .map((model) => ({ model, score: scoreModel(model, normalized) }))
    .filter((entry) => entry.score > 0)
    .sort((left, right) =>
      right.score - left.score ||
      left.model.modelName.localeCompare(right.model.modelName),
    )
    .slice(0, 80)
    .map((entry) => entry.model);
}

export function formatModelLabel(model: BestdoriModelEntry): string {
  return model.costumeDescriptions[3] || model.costumeDescriptions[1] || model.costumeDescriptions[0] || "未命名服装";
}

export function getBestdoriAssetBase(): string {
  return ASSETS_BASE;
}

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(noCacheUrl(url), { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`请求失败：${url} HTTP ${response.status}`);
  }
  return response.json() as Promise<T>;
}

function readNestedRecord(source: Record<string, unknown>, keys: string[]): Record<string, unknown> {
  let current: unknown = source;
  for (const key of keys) {
    if (!current || typeof current !== "object") return {};
    current = (current as Record<string, unknown>)[key];
  }
  return current && typeof current === "object" ? current as Record<string, unknown> : {};
}

function compactStrings(values?: Array<string | null>): string[] {
  return (values ?? []).filter((value): value is string => Boolean(value?.trim()));
}

function normalizeSearchText(value: string): string {
  return value.normalize("NFKC").trim().toLowerCase().replace(/\s+/g, "");
}

function scoreModel(model: BestdoriModelEntry, query: string): number {
  const fields = [
    model.modelName,
    model.characterId,
    ...model.characterNames,
    ...model.costumeDescriptions,
  ].map(normalizeSearchText);

  let score = 0;
  for (const field of fields) {
    if (!field) continue;
    if (field === query) score = Math.max(score, 100);
    else if (field.startsWith(query)) score = Math.max(score, 70);
    else if (field.includes(query)) score = Math.max(score, 40);
  }
  if (model.modelName.toLowerCase() === query) score += 20;
  return score;
}
