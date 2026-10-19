## Why

qwen3-1.7b 在 JSON 转义/去转义回合里已经能选对工具，但上下文一重置就丢掉工具结果和报错，于是重复调用，或改用示例里的短 JSON。`json.escape` 遇到多一层反斜杠时只报 `key must be a string`，模型不会改回用户原文。常驻的上百个工具目录也让小模型抄示例、一次发出两个相同调用。

## What Changes

- 上下文重置时保留本轮用户原文，以及最近一对工具调用和结果（过长的成功结果只留开头）。
- `json.escape` 先按原文转义；原文不是合法 JSON 时剥掉多余反斜杠再转义。仍然失败时，错误里必须包含「不要再多写反斜杠」。弯引号先归一成 ASCII 引号。
- `json.format` 的参数说明只描述美化，不再写「反转义」。
- 小模型系统提示在检索命中技能时，只列出这些技能登记的工具。
- 同一轮里工具名和参数完全相同的调用只执行一次。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `agent-tool-harness`：重置保留最近工具交换；小模型工具目录按检索收窄；同参调用去重；`json.escape` 对多余反斜杠给出可重试错误或直接剥层成功。

## Impact

- `src-tauri/src/agent/compress.rs`、`loop.rs`、`harness/prompt.rs`、`registry.rs`
- `src-tauri/src/commands/json.rs`（`escape_json`，页面转义与 agent 共用）
