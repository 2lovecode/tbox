## Why

聊天界面虽已有思考折叠、工具卡片与分块 `token`/`reasoning` 事件，但仍存在明显体验缺口：工具调用在回合结束后（及历史重开）常不可见；步骤顺序未作为统一「轨迹」展示；模型侧多为整段生成后再 `chunk_text` 假流式，长等待后才突然刷字。用户需要在对话中按时间线看到思考、工具与正文，并在支持的后端上真正边生成边推送。

## What Changes

- Agent 循环改为优先走 `ChatModel::complete_streaming`：真流式后端边采样/边 SSE 推送 delta；不支持流式的协议 **降级** 为整段完成后分块，并通过 `stream_meta` 标明 `live` / `fallback`。
- 前端以 **助手回合时间线** 按序展示：思考 → 工具调用（参数/结果）→ 正文；流式期间全部展开，结束后思考与工具默认收起、正文展开；历史重开保持同一顺序。
- 助手消息持久化新增有序 `trajectory_json`（与既有 `content` / `reasoning` / `tool_calls_json` 并存）；旧消息无轨迹时由现有字段合成。
- 增补 `agent-event`：`stream_meta`；其余事件语义保持，前端改为按轨迹累积而非「工具区与气泡分离」。

### Non-goals

- 步骤级单独复制、导出整条轨迹、单步重跑工具。
- 用户消息编入时间线。
- 首期不保证多模态/特殊 content block 的流式展示（仅 text + reasoning + tool_calls）。
- 不改变工具白名单、Skill 注入、harness 修复预算等 Agent 执行语义。

## Capabilities

### New Capabilities

- （无）本 change 不新增独立 capability 目录；行为落在既有 chat / reasoning 能力上。

### Modified Capabilities

- `agent-chat`: 助手回合以有序轨迹展示思考/工具/正文；历史重开可见；流式与结束后的展开规则；`fallback` 轻提示。
- `chat-reasoning-display`: 从「仅折叠思考块」扩展为轨迹中的思考步骤；真流式与降级分块均须经 reasoning/token 事件；持久化含轨迹顺序。

## Impact

- 前端：`src/views/HomePage.vue` 及新建时间线组件；`conversations` store / 消息类型。
- Rust：`agent/llm.rs`（`ChatModel` 流式接口）、`agent/loop.rs`、`embedded_engine.rs`、`genai_model.rs`、OpenAI 兼容客户端；`commands/conversation.rs`（migration + `trajectory_json`）；`commands/agent.rs` 事件下发。
- 事件契约：`agent-event` 新增 `stream_meta`（非 BREAKING：旧前端可忽略未知类型）。
- 评测 / mock：`complete` 保留；streaming 增加单测与可选 eval 覆盖。
