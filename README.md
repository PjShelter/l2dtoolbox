# l2dtoolbox

`l2dtoolbox` 是一个面向 Windows 的 `Vue + Tauri` 桌面工具，用来替代旧的 `PySide + pygame/live2d-py` 方案，统一处理：

- `model.json` 批处理
- `.jsonl` 生成、编辑、规范化
- Live2D 单模型与组合模型预览
- 本地工作台配置持久化

## 技术栈

- `Vue 3 + Vite`
- `Tauri 2`
- `pixi.js`
- `pixi-live2d-display-webgal`
- `composite-model`

## 当前模块

1. `模型工具`
   - 扫描目录生成 `model.json`
   - 清理重复动作 / 表情和缺失资源
   - 批量导入 `.mtn` / `.exp.json`
   - 批量修改 / 删除 `.mtn` 参数

2. `JSONL 工作台`
   - 读取 `.jsonl`
   - 编辑 `parts` 与 `summary`
   - 规范化并保存
   - 解析预览资源路径

3. `预览`
   - 单模型 `model.json` 预览
   - 组合 `.jsonl` 预览
   - 读取动作 / 表情列表
   - 统一背景和视口控制

## 本地运行

```bash
pnpm install
pnpm tauri:dev
```

仅构建前端资源：

```bash
pnpm build
```

运行测试与检查：

```bash
pnpm test
cd src-tauri && cargo test && cargo check
```

## 资源说明

- `lib/live2d.min.js`
- `lib/live2dcubismcore.min.js`

这两个运行库会在前端预览模块里动态注入，用于 Cubism 运行时初始化。
