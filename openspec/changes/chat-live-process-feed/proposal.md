## Why

聊天回合处理中，前端目前只显示一条 shimmer 状态（「思考中…」），过程摘要列表被默认收起；规格要求流式期间展示过程步骤。用户希望接近 Cursor：处理中逐条刷出简略动作，结束后整块收进可点开的过程区。

## What Changes

- 流式进行中：过程区默认展开，思考/工具以一行摘要逐条出现；参数/结果默认不展开，点行才看细节。
- 回合结束后（及历史重开）：过程区默认收起为「思考了 Xs」；点开可看完整摘要。
- 正文仍在过程区下方流式/静态展示；不改后端事件与轨迹持久化。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-chat`: 明确「简略过程 feed + 结束后收起」的 UI 行为（对齐并细化既有 Assistant Turn Trajectory Display）。

## Impact

- 前端：`AssistantTrajectory.vue`（及少量样式）
- 规格：`openspec/specs/agent-chat`
- 无 Rust / 契约变更
