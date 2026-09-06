## Why

聊天区身份装饰（头像、助手名）与强对比用户气泡增加视觉噪声；默认窗口过大；复制按钮常显不够克制。需要收敛为更干净的对话阅读体验。

## What Changes

- 默认主窗口尺寸缩小（约 1200×780），并保证默认宽度下仍为侧栏+主区双栏。
- 聊天界面移除用户头像、助手头像与助手显示名；设置中移除用户头像预设与助手显示名配置。
- 仅用户消息使用气泡样式，且为弱强调（淡底/细边），助手正文为无强气泡的纯文本块。
- 用户与助手正文的复制按钮改为悬停（或焦点）时出现在对应消息下方。
- Non-goals：不改工具过程轨复制；不改 Agent 循环；不改 LLM 引擎日志设置。

## Capabilities

### New Capabilities

- （无）

### Modified Capabilities

- `agent-chat`: 紧凑默认窗口；无头像/助手名；弱用户气泡；复制悬停于消息下方；废止 Agent Avatar Variety。

## Impact

- `tauri.conf.json` 窗口宽高；`HomePage.vue` / `AssistantTrajectory.vue`；`GeneralSettingsPanel.vue` 去掉身份表单；可删除 `useChatIdentity`。
- 无 Rust 推理契约变更。
- 说明：`settings-general` 身份要求仅存在于未归档的 `chat-identity-copy`；本 change 通过删除实现与 UI 收回，不另建 settings-general delta。
