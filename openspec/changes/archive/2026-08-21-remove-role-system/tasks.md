## 1. Backend：删除角色命令与表

- [x] 1.1 从 `src-tauri/src/commands/tool.rs` 的新库初始化中删除 `roles` / `tool_roles` 建表与 seed，并在已有库迁移路径加入 `DROP TABLE IF EXISTS tool_roles` 再 `DROP TABLE IF EXISTS roles`。用 `rg "CREATE TABLE IF NOT EXISTS roles|INSERT INTO roles|INSERT INTO tool_roles" src-tauri` 确认无命中。
- [x] 1.2 删除 `src-tauri/src/commands/role.rs`，从 `commands/mod.rs` 与 `lib.rs` 去掉 `role` 模块及 `get_roles` / `get_tools_by_role` / `set_user_role` / `get_user_role` 注册。运行 `cd src-tauri && cargo check` 通过。
- [x] 1.3 清理 `search.rs` 等处仅文档性的角色加权注释（命令本身保持角色无关）。用 `rg "get_tools_by_role|get_roles|set_user_role|get_user_role" src-tauri` 确认无命中。

## 2. Frontend：首页、侧边栏与 store

- [x] 2.1 从 `HomePage.vue` 移除 `RoleSelection`、角色过滤与「显示全部 / 仅显示我的角色」开关；启动后直接渲染全部已注册工具。用 `rg "RoleSelection|showAllTools|useRoleStore" src/views/HomePage.vue` 确认无命中。
- [x] 2.2 从 `SideBar.vue` 移除角色过滤；分类计数基于 `store.tools` 全量。用 `rg "useRoleStore|roleToolIds|showAllTools" src/layout/SideBar.vue` 确认无命中。
- [x] 2.3 删除 `src/stores/role.ts`、`src/types/role.ts`、`src/components/onboarding/RoleSelection.vue`。用 `rg "stores/role|types/role|RoleSelection" src` 确认无引用。

## 3. Frontend：设置与 Spotlight

- [x] 3.1 从 `SettingsModal.vue` 删除角色 tab 与相关逻辑；`settings.ts` 的 `SettingsTab` 不再包含 `'role'`，默认 tab 为 `'llm'`。打开设置可见 LLM 页且无「角色」页（代码审查 + `rg "role" src/stores/settings.ts src/components/settings/SettingsModal.vue` 无角色业务命中；ARIA `role=` 除外）。
- [x] 3.2 从 `SpotlightSearch.vue` 删除角色加权、范围过滤与角色 chips；保留快捷键、键盘导航、拼音/分词与 AI 开关。用 `rg "useRoleStore|get_tools_by_role|roleToolBoost|_roleBoost" src/components/SpotlightSearch.vue` 确认无命中。

## 4. 文档与规格索引

- [x] 4.1 更新 `README.md` 中角色 API / 引导说明，改为首页展示全部工具。用 `rg "get_tools_by_role|useRoleStore|RoleSelection|user_roles" README.md` 确认无过时说明。
- [x] 4.2 从 `AGENTS.md` living specs 表移除 `roles` 行（归档时再删 `openspec/specs/roles/`）。确认表中不再列出 `openspec/specs/roles/`。

## 5. 验证

- [x] 5.1 运行 `cd src-tauri && cargo check` 与前端 `npx vue-tsc --noEmit`（或项目现有类型检查脚本），两者通过。
- [x] 5.2 全库检索 `useRoleStore`、`get_tools_by_role`、`get_roles`、`set_user_role`、`get_user_role`、`tool_roles` 仅允许出现在 OpenSpec change / 历史 archive 文档中，业务代码无命中。
