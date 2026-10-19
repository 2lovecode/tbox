## Why

小模型要在很多工具里选调用。Laya 已经能把一句话路由到工具箱页面，但只用于 Spotlight。对话里可以把它当成高置信度参考，同时仍由模型决定工具和参数。

## What Changes

- 发给 Laya 之前，用规则把本轮决策状态压短：去掉 JSON 等载荷，短跟进句附上一轮工具名和一行结果。
- Laya 未启用、超时、模型不可用或低于阈值时，Agent 行为与现在相同。
- 命中时在系统提示中加一条可忽略的参考（页面名 + 置信度）。该页面对应的 agent 工具排到目录前面，其余已检索工具仍保留。
- 不按 Laya 的页面 id 直接派发，不改写用户原文。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `agent-tool-harness`：小模型回合可附带 Laya 参考，参考不是强制工具选择。

## Impact

- `src-tauri/src/agent/loop.rs`、`harness/prompt.rs`、`skills.rs`
- `src-tauri/src/commands/laya.rs`（同步决策入口，Spotlight 行为不变）
