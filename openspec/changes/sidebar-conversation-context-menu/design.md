## Context

侧栏 `SideBar.vue` 历史列表左键打开会话，悬停显示删除按钮；改名与轨迹入口在 `HomePage` 聊天顶栏。既有 store 已提供 `renameConversation` / `deleteConversation`，路由 `/agent-runs/:id` 已可用。本 change 仅改前端交互壳层。

## Goals / Non-Goals

**Goals:**
- 右键打开轻量浮层菜单：改名、打开轨迹、删除
- 去掉悬停删除按钮，降低误触
- 视觉与现有 shell token（`--bg-*` / `--surface-*`）一致

**Non-Goals:**
- 通用 ContextMenu 组件 / 系统原生菜单
- 置顶、批量操作、新后端 API

## Decisions

1. **实现位置**：逻辑与模板全部放在 `SideBar.vue`（Teleport 到 `body` 可选，优先同组件内 fixed 定位）。  
   - 备选：抽通用组件 → 本期过重，拒绝。

2. **改名**：菜单「改名」→ 该行进入 input；Enter / blur 调用 `renameConversation`；Esc 取消。空标题不提交。  
   - 备选：单独弹窗 → 打断流，拒绝。

3. **打开轨迹**：`router.push(/agent-runs/${id})`，与顶栏一致；不强制先 `openConversation`。

4. **删除**：`window.confirm` 后调用 `deleteConversation`；菜单关闭。  
   - 备选：无确认 → 误触成本高；自定义 modal → 本期过重。

5. **菜单关闭**：点击外部、Esc、滚动侧栏、选择项后关闭；`contextmenu.prevent` 阻止浏览器默认菜单。

6. **定位**：以鼠标 clientX/Y 为锚点；靠近视口右/下边缘时翻转，避免裁切。

## Risks / Trade-offs

- [滚动时浮层错位] → 打开后监听侧栏 scroll / window 即关闭  
- [confirm 样式不统一] → 接受原生 confirm，后续可换  
- [无悬停删除后 discoverability] → 依赖右键；可接受（与常见聊天产品一致）

## Migration Plan

- 纯前端；发版即可。回滚：恢复悬停删除、去掉 contextmenu。

## Open Questions

- （无）删除确认采用原生 `confirm`（已定）
