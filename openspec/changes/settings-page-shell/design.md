## Context

现有设置由 `SettingsModal.vue` + `useSettingsStore`（`isOpen` / `activeTab`）驱动，在 `App.vue` 全局挂载；LLM 列表与表单同屏，提供方在预设较少时用 `<select>`，筛选列表仅在过滤时出现。见 proposal.md - Why。后端 profile CRUD（`useLlmStore` / Tauri commands）保持不变。

## Goals / Non-Goals

**Goals:**
- 路由化设置页 + 左菜单壳，可扩展分区
- LLM 区：列表与编辑分离；新建提供方选择改为筛选 + 展开列表；编辑时提供方只读
- 所有入口统一为页面导航

**Non-Goals:**
- 不改 Rust 侧配置存储 / 加密 / 模型下载命令
- 不实现「通用」「关于」实质功能
- 不做设置页视觉体系大改（沿用现有按钮 / 表单样式）

## Decisions

### 1. 路由：`/settings` + 可选 section 段
- **选择**：`/settings` 默认 LLM；`/settings/:section` 支持 `llm` | `general` | `about`；非法 section 回退 `llm`。
- **理由**：可深链、可刷新保持分区；与现有 vue-router 一致。
- **备选**：仅 query `?tab=` — 刷新可用但不如 path 清晰；拒绝保留 modal + 同步路由（双入口复杂）。

### 2. Store：去掉弹窗态，保留/改成 section 偏好（可选）
- **选择**：`useSettingsStore` 移除 `isOpen`；入口改为 `router.push('/settings' | '/settings/llm')`。`activeTab` 可删，以路由为真相；若需「记住上次分区」可后续再加，本变更默认进 LLM。
- **理由**：页面态不需要 modal 开关。

### 3. 组件拆分
- **选择**：
  - `views/SettingsPage.vue`：壳（左菜单 + `<RouterView>` 或动态面板）
  - `components/settings/LlmSettingsPanel.vue`：列表 + 打开对话框
  - `components/settings/ProfileEditorDialog.vue`：新建/编辑表单（从现有 modal 表单迁出）
  - `components/settings/ProviderPicker.vue`：筛选 + 展开列表
  - 占位：`GeneralSettingsPanel` / `AboutSettingsPanel`（极简文案）
- **理由**：单文件 `SettingsModal` 已过大，拆分便于维护。
- **备选**：整页一个 Vue 文件 — 短期更快，但违反可维护性。

### 4. 编辑只读提供方
- **选择**：编辑模式禁用 `applyPreset`；UI 展示 label，不渲染可点选列表。
- **理由**：产品约定「换提供方请新建」，避免半迁移 Key/协议歧义。

### 5. App 壳与侧栏
- **选择**：`/settings` 与工具页一样不显示工具 `SideBar`；设置页内部自带左菜单。
- **理由**：避免双导航冲突。

### 6. 本地模型下载 / Ollama pull UI
- **选择**：仍放在新建/编辑对话框内（选 local / ollama 时），逻辑从现有 modal 迁移，不丢能力。
- **理由**：规格要求配置字段语义不变；列表页保持干净。

## Risks / Trade-offs

- [对话框承载本地模型下载进度] → 关闭对话框时 MUST 不取消后台下载；重新打开编辑/新建时仍能看到进度（沿用现有事件监听，挂到 panel 或 dialog 生命周期）。
- [深层表单迁出易漏字段] → 对照现有 `SettingsModal` 清单迁移，保存/测试/拉取模型按钮行为做一次手工核对。
- [快捷键 toggle 语义变 navigate] → `Cmd+,` 若已在设置页可 `router.back()` 或保持停留；推荐：已在 `/settings*` 则忽略或回到上一页，避免「关不掉」错觉——实现定为：已在设置页再按快捷键则 `router.back()`（无历史则回 `/`）。

## Migration Plan

1. 落地路由与空壳页 → 2. 迁移 LLM 列表 → 3. 对话框 + ProviderPicker → 4. 改入口、删除 `SettingsModal` → 5. 手工验收入口与 CRUD。
回滚：恢复 modal 组件与入口即可；无数据迁移。

## Open Questions

无（产品选择已在设计对话中确认：占位菜单、默认 LLM、编辑提供方只读）。
