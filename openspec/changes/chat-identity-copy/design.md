## Context

See proposal.md - Why。聊天轨迹 UI（`AssistantTrajectory`）已区分过程轨与助手正文；用户消息在 `HomePage` 无头像；复制仅部分消息有悬停按钮；`GeneralSettingsPanel` 仍为占位。主题等偏好已用前端 `localStorage`（如 `useTheme`），本变更沿用该模式，不引入 Rust settings API。

## Goals / Non-Goals

**Goals:**
- 统一复制交互组件/样式，覆盖用户正文、助手正文、工具入参/出参。
- 用户头像预设 + 助手显示名全局偏好，设置「通用」可改，对话即时反映。
- 与既有会话随机助手头像规格并存。

**Non-Goals:**
- 自定义图片上传、按会话覆盖助手名、思考步骤复制、后端 SQLite 偏好表。

## Decisions

### 1. 偏好存储：前端 localStorage composable
- **选择**：`useChatIdentity`（或等价 store）读写 `tbox.userAvatarId` / `tbox.assistantName` 键；默认头像 id 与默认名常量写死在前端。
- **替代**：SQLite `app_settings` 表 — 过重且无跨端同步需求；本期不做。
- **理由**：与主题一致、零后端改动、满足重启持久化。

### 2. 用户头像：Font Awesome / 既有图标预设集
- **选择**：约 6–12 个预设（图标类名 + 背景色），设置页网格点选；对话用同一映射渲染。
- **替代**：emoji 字符 — 跨平台渲染不一致；本期用图标更贴合现有助手头像实现。

### 3. 助手名展示位置
- **选择**：助手正文行头像旁增加小号显示名（过程轨不加名，避免噪音）。
- **替代**：每条过程步骤都带头像名 — 拒绝，保持过程轨弱视觉。

### 4. 复制 UI
- **选择**：抽出轻量 `CopyIconButton`（或局部复用同一 class）：hover/focus-within 显示，`navigator.clipboard.writeText`，keyed 成功态 ~1.5s。工具入参复制 `JSON.stringify(args, null, 2)`；出参复制 `result` 字符串。
- **替代**：始终可见文字「复制」— 用户已选方案 A（图标 + 悬停）。

### 5. 设置表单提交
- **选择**：头像点选即写；显示名 `change`/`blur` 时写（空则回退默认）。无需单独「保存」按钮，降低摩擦。

## Risks / Trade-offs

- [多 WebView / 未来多窗口不同步] → 可接受；单窗 Tauri 主场景足够。
- [助手名过长撑破布局] → CSS 截断 + title 悬停全文。
- [工具入参非 JSON 可序列化] → stringify 失败时回退 `String(args)`。

## Migration Plan

- 无 DB migration。首次启动无键 → 默认预设与默认名。
- 回滚：移除 UI 与 composable 即可；localStorage 键可残留无害。
