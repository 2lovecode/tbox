## Why

当前设置以全屏遮罩弹窗呈现，LLM 配置列表与表单挤在同一视图，提供方选择依赖下拉框，扩展更多设置项时难以承载。需要独立设置页与清晰分区，并改善提供方配置的创建/编辑体验。

## What Changes

- **BREAKING（交互）**：设置从弹窗改为独立路由页面；顶栏齿轮、`Cmd/Ctrl+,` 及「去设置」入口改为进入该页面，不再打开 `SettingsModal`。
- 设置页采用左侧菜单 + 右侧内容布局；默认进入「LLM 配置」。
- 左侧菜单首期包含：LLM 配置、通用（占位）、关于（占位）。
- LLM 配置区仅展示已保存的提供方 profile 列表；新建与编辑改为对话框。
- 新建时提供方选择为「筛选 + 全部展开列表」（不再用下拉）；编辑时提供方只读，换提供方需新建。
- 退役 `SettingsModal` 弹窗壳；profile CRUD / 密钥加密等后端契约不变。

## Capabilities

### New Capabilities

- `settings-page`: 独立设置页壳（路由、左菜单、分区占位）以及 LLM 配置区的列表 / 新建编辑弹窗 / 提供方展开选择交互。

### Modified Capabilities

- （无）`llm-provider-profiles` 的存储与激活语义不变；本变更仅改变前端呈现与入口。

## Impact

- 前端：`src/router/`、`src/App.vue`、`src/stores/settings.ts`、`src/components/settings/*`、`src/views/SettingsPage.vue`（新建）、`ModelSwitcher` 等入口。
- 无新 Rust command、无新工具 id；不改 SQLite 工具注册。
- Non-goals：本变更不实现「通用 / 关于」实质配置项；不改多 profile 后端存储与加密；不做设置页深色主题专项 redesign。
