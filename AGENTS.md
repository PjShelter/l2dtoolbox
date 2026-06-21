# AGENTS.md

Compact guidance for OpenCode sessions working in `l2dtoolbox`.
See also `CLAUDE.md` for the original architecture overview.

## Commands

```bash
pnpm install              # required before anything (frozen-lockfile in CI)
pnpm tauri:dev            # full app: Vite on :1420 + Tauri shell
pnpm build                # typecheck (vue-tsc --noEmit) + Vite build
pnpm test                 # vitest run (all tests)
pnpm test src/lib/manifest.test.ts   # single file

cd src-tauri && cargo check   # backend typecheck
cd src-tauri && cargo test    # backend tests
```

Non-obvious command facts:
- **No `lint` or separate `typecheck` script.** `pnpm build` is the only typecheck gate for the frontend. Run it (or `vue-tsc --noEmit`) before declaring frontend work done.
- `pnpm tauri:dev` runs `pnpm dev` itself via `tauri.conf.json` `beforeDevCommand`; do not start Vite separately.
- Vite port `1420` is `strictPort: true` — a port collision will hard-fail, not auto-increment.
- `pnpm-workspace.yaml` only sets `allowBuilds: esbuild`; this is **not** a multi-package workspace. Treat the repo as a single package.

## Wiring: adding a new Tauri command

A new backend operation touches **four** places — missing any one silently breaks the call:

1. Implement the fn in `src-tauri/src/{model,jsonl,settings}.rs`.
2. Add a `#[tauri::command]` wrapper in `src-tauri/src/lib.rs` that delegates and maps `anyhow::Error -> String`.
3. Register the name in the `tauri::generate_handler![...]` list in `lib.rs:run()`.
4. Add a typed wrapper in `src/lib/tauri.ts` (frontend calls only go through this file).

## Type mirror requirement

`src-tauri/src/types.rs` and `src/types/app.ts` must serialize to the **same JSON shape**. All Rust structs use `#[serde(rename_all = "camelCase")]` to match TS interfaces. When changing a shared type, update both files or `invoke()` will silently mis-deserialize.

## Backend quirks

- `src-tauri/resources/parts.json` is embedded at compile time via `include_str!` in `model.rs`. Editing it requires `cargo` rebuild; it is not read from disk at runtime.
- `settings.rs` persists `AppSettings` to Tauri's `app_data_dir()/settings.json`. No env vars.
- The part-editor / init-state writer (`write_model_init_state`) only supports **Cubism 2 `model.json`**; it bails on `.model3.json`. `scan_preset_targets` also filters via `is_cubism2_model_json`.
- `tauri.conf.json` has `csp: null` and `assetProtocol.scope: ["**"]` — any local file is reachable. Do not tighten this without updating every `toAssetUrl()` consumer.
- Capabilities in `src-tauri/capabilities/default.json` cover both `main` and `preview-window` windows.

## Frontend quirks

- **Live2D Cubism runtime libs (`public/lib/live2d.min.js`, `live2dcubismcore.min.js`) must not be bundled by Vite.** They are injected as raw `<script>` tags by `src/lib/runtime/cubism.ts` before any model loads. Do not import them from TS.
- `composite-model` (npm) owns `.jsonl` parsing on the frontend. The Rust side has its own independent parser in `jsonl.rs` — keep the two in sync when the format changes.
- `.jsonl` format = part-entry lines (layer defs) + one summary line (motions/expressions + import index). `CompositeManifest` carries both plus diagnostics.
- Use `toAssetUrl(filePath)` from `lib/tauri.ts` (wraps `convertFileSrc` + backslash normalization) for **every** local path handed to `<img>` or the Live2D loader.
- All `invoke()` calls go through `lib/tauri.ts`. Pages/components should not call `invoke` directly.
- `.gitignore` excludes root `/lib/`; the tracked runtime libs live at `public/lib/`. Don't confuse the two.

## Tests

- Vitest config lives in `vite.config.ts` with `environment: "node"` (no DOM). Tests that need `window`/`document` will fail unless the environment is changed.
- Currently only `src/lib/manifest.test.ts` exists. There is no frontend test for Tauri-bound code; the Rust side has unit tests in `jsonl.rs` / `model.rs` run via `cargo test`.
- CI (`.github/workflows/release.yml`) **does not run tests** — it only builds. Tests are a local responsibility.

## Release

- Windows-only. Tag push matching `v*` triggers `tauri-apps/tauri-action` to build and publish a GitHub release.
- `workflow_dispatch` builds the bundle (`pnpm tauri build`) and uploads `msi`/`nsis` artifacts, but does **not** publish a release.
- No other platforms are configured; do not add macOS/Linux jobs without confirming the Live2D runtime + asset protocol behavior.
