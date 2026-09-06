## Why

聊天区复制能力不完整（工具入参/出参、助手正文体验不一致），用户消息缺少头像，助手亦无可配置显示名，观感不像完整对话产品。需要在现有轨迹布局上补齐身份展示与统一复制交互，并在设置「通用」分区落地可改偏好。

## What Changes

- 用户消息展示预设头像（可在设置中更换）；助手消息保留每会话随机头像，并显示可配置的助手显示名。
- 统一复制动作条：用户消息正文、助手正文、工具入参、工具出参均可复制；悬停显示图标按钮，成功后短暂变为勾选反馈。
- 设置页「通用」分区从占位改为实质表单：用户头像预设选择、助手显示名编辑；偏好本地持久化（与主题类似的前端存储即可）。
- Non-goals：不上传自定义头像图；不改变助手「每会话随机头像」规则；不为思考过程步骤提供复制；不改 Agent 循环 / LLM / 工具执行语义。

## Capabilities

### New Capabilities

- `settings-general`: 设置页「通用」分区的聊天身份偏好（用户头像预设、助手显示名）及其持久化与默认值。

### Modified Capabilities

- `agent-chat`: 对话气泡身份栏（用户头像、助手名+头像）与统一复制交互（含工具入参/出参）。

## Impact

- 前端：`HomePage.vue`、`AssistantTrajectory.vue`、`GeneralSettingsPanel.vue`、新建偏好 composable/store（localStorage）、复制按钮样式统一。
- 无新 Rust command、无新工具 id、无 SQLite schema 变更（偏好走前端 localStorage）。
- 与既有 `Agent Avatar Variety` 并存：会话随机助手头像不变；新增的是全局助手**显示名**与用户头像预设。
