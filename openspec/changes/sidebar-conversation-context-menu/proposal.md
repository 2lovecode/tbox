## Why

侧栏历史会话目前只有悬停删除；改名在聊天顶栏、轨迹在顶栏入口，操作分散且悬停按钮易误触。需要把会话常用操作集中到右键菜单，并去掉悬停删除。

## What Changes

- 侧栏历史会话项支持右键上下文菜单：**改名**、**打开轨迹**、**删除**
- **移除**会话行悬停显示的删除按钮
- 改名在列表项就地编辑，复用既有 `rename_conversation` / `renameConversation`
- 打开轨迹导航至既有 `/agent-runs/:id`
- 删除复用既有 `delete_conversation`；删除前 `confirm` 防误触
- 无新 Rust API、无 **BREAKING** 契约变更

### Non-goals

- 不引入通用 ContextMenu 组件库或 Tauri 原生系统菜单
- 不新增置顶、复制标题、多选批量删除等能力
- 不改会话列表排序 / 存储模型

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-chat`: 会话历史交互——侧栏右键菜单为改名 / 打开轨迹 / 删除的入口；不再依赖悬停删除按钮

## Impact

- 前端：`src/layout/SideBar.vue`（主改）；可能轻触 `stores/conversations.ts`（仅复用既有 rename/delete）
- 路由：复用 `/agent-runs/:id`（与 `HomePage` 顶栏轨迹一致）
- 后端：无新 command
