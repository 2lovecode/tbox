## Context

见 `proposal.md` 的 Why。当前角色能力跨前端 store、引导页、设置、首页/侧边栏过滤、Spotlight 加权，以及 SQLite `roles` / `tool_roles` 与 Tauri 命令。拆除必须一次切干净，避免留下半套死代码。

约束：工具分类与 `product` 目标用户文案保持不变；搜索的快捷键、键盘导航、拼音/分词保留。

## Goals / Non-Goals

**Goals:**

- 删除角色相关 UI、状态、命令与表结构，使启动路径不再依赖角色。
- 已有本地库能在启动迁移中丢掉 `roles` / `tool_roles`，新库不再创建它们。
- 前端编译与 `cargo check` 在删除命令后仍通过。

**Non-Goals:**

- 不引入替代性的「工具推荐 / 个性化」机制。
- 不扫描或删除用户机器上的遗留配置文件。
- 不改工具 id、分类、标签或路由。

## Decisions

### 1. 物理删除模块，而不是把过滤条件设为「全部」

选择删除 `role.rs`、`useRoleStore`、引导组件与设置 tab。  
备选：保留表和命令、前端永远传空角色。否决原因：死代码仍要随工具注册维护 `tool_roles` 映射。

### 2. 启动时 DROP 表，而不是留下空表

在现有 `tool.rs` 迁移路径（与 `add_missing_tools` 同类的启动迁移）中执行 `DROP TABLE IF EXISTS tool_roles;` 再 `DROP TABLE IF EXISTS roles;`（先子表后父表）。新库初始化代码删除建表与 seed。  
备选：只停写、保留表。否决原因：与「全都去掉」不一致，后续开发者会误以为还要填角色映射。

### 3. 遗留配置不主动删除

`tbox.role.selection`（localStorage）与 `%config%/tbox/user_roles.json` 停止读写即可。  
备选：启动时 unlink 文件并清 localStorage。否决原因：收益低，且桌面应用清理用户目录需额外权限与失败处理。

### 4. 设置默认 tab 改为 LLM

`SettingsTab` 从 `'role' | 'llm'` 变为仅 `'llm'`（若日后只有一页，可去掉 tab 栏，但本 change 只保证角色页消失）。  
备选：留空的角色 tab。否决原因：空 tab 比没有更糟。

### 5. Spotlight 去掉角色加权后沿用现有非角色排序

删除 `get_tools_by_role`、role boost、结果上的角色 chips。拼音/分词/AI 开关逻辑不动。  
备选：用分类 chips 替换角色 chips。否决原因：超出本 change；结果项已有工具自身信息。

### 6. Archive 后删除空的 `roles` living spec

delta 将 `roles` 三条要求全部 REMOVED。归档时若主 spec 不再含 Requirements，应删除 `openspec/specs/roles/`，并更新 `AGENTS.md` living specs 表。  
备选：留一个空 capability。否决原因：会误导后续 change。

## Risks / Trade-offs

- [已发布前端仍 invoke 已删命令] → 本仓库前后端同发；无独立 API 兼容窗口。`cargo check` + 前端类型检查覆盖调用点。
- [旧库 DROP 失败导致启动报错] → 使用 `IF EXISTS`；`tool_roles` 先于 `roles` 删除以避开 FK。
- [部分工具过去只挂在某一角色上，用户误以为工具被删] → 实际是全部展示；用首页全量列表验证，不另做迁移提示。

## Migration Plan

1. 发布含拆除代码的版本。
2. 用户首次启动：迁移 DROP 两张表；前端不再读角色 store。
3. 回滚：回退到上一版本；若表已 DROP，旧版会在其「表不存在则重建」路径中重新 seed（旧 `tool.rs` 已有该逻辑）。本 change 不提供专门 rollback 脚本。

## Open Questions

无。
