## Why

TBox 当前用「个人开发者 / 团队 / 企业」三档角色做首次引导、首页过滤和 Spotlight 加权，实际把同一套工具箱切成不完整子集，增加上手成本却没有对应的产品价值。现在去掉这套区分，让所有用户直接看到完整工具列表。

## What Changes

- **BREAKING**：移除首次启动的角色选择引导；用户进入应用即可使用首页。
- **BREAKING**：首页与侧边栏不再按角色过滤工具，去掉「显示全部 / 仅显示我的角色」开关。
- **BREAKING**：设置中移除「角色」页；默认进入 LLM 等非角色设置。
- **BREAKING**：Spotlight 不再按角色过滤、加权或展示角色标签。
- **BREAKING**：删除 Tauri 命令 `get_roles`、`get_tools_by_role`、`set_user_role`、`get_user_role`。
- 新库不再创建 `roles` / `tool_roles` 表；已有库在启动迁移中删除这两张表。
- 删除前端 `useRoleStore`、`RoleSelection` 与角色类型定义。

## Capabilities

### New Capabilities

- （无）

### Modified Capabilities

- `roles`：整份能力移除（角色持久化、首次引导、按角色过滤工具）。
- `spotlight`：移除「角色感知搜索」要求。
- `local-ai-search`：本地智能搜索不再包含角色感知意图路由。

## Impact

**前端页面 / 组件**

- `src/views/HomePage.vue`：去掉引导遮罩、角色过滤与「显示全部」开关。
- `src/layout/SideBar.vue`：分类计数改为基于全部已注册工具。
- `src/components/SpotlightSearch.vue`：去掉角色加权、范围过滤与角色 chips。
- `src/components/settings/SettingsModal.vue` 与 `src/stores/settings.ts`：删除角色 tab。
- 删除 `src/components/onboarding/RoleSelection.vue`、`src/stores/role.ts`、`src/types/role.ts`。

**Rust commands**

- 删除 `src-tauri/src/commands/role.rs` 及在 `lib.rs` / `commands/mod.rs` 中的注册。
- `src-tauri/src/commands/tool.rs`：初始化与迁移不再建/填 `roles`、`tool_roles`；已有库 DROP 这两张表。

**其它**

- 浏览器 `localStorage` 键 `tbox.role.selection` 与配置目录 `user_roles.json` 不再读取；不强制清理遗留文件。
- 文档：`README.md`、`AGENTS.md` living specs 表中的角色说明需同步删除或改写。
- 无新 npm/crate 依赖；无工具 id 变更。

## Non-goals

- 不改工具分类（编码、网络、数据库等）与首页按分类浏览。
- 不改 `product` 规格中的目标用户定位文案（后端 / 测试 / 运维）。
- 不改 Spotlight 全局快捷键、键盘导航、拼音/分词等非角色搜索能力。
- 不强制删除用户机器上已存在的 `user_roles.json` 或 localStorage 残留键。
