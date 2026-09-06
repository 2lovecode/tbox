## Context

See proposal.md — Why。当前链路为 `ChatModel::complete` 整段返回 → `loop` 用 `chunk_text` 假流式 emit → 前端 `HomePage` 分 `pendingTools` 与气泡两套渲染；历史消息不渲染 `tool_calls_json`。约束：保留 harness 校验/reask、工具白名单、既有 `agent-event` 频道；SQLite 向后兼容 migration；评测可继续走非流式 `complete`。

## Goals / Non-Goals

**Goals:**

- 统一 `complete_streaming` + 默认 fallback，loop 只消费流式入口。
- 有序 `trajectory_json` 持久化与前端时间线组件。
- 嵌入引擎与 OpenAI 兼容 SSE 优先真流式；其余协议尽力，否则 fallback + UI 提示。

**Non-Goals:**

- 步骤复制/重跑、用户消息入轨迹、多模态流式。
- 为每种云端协议手写完整 SSE 解析器若 genai 已提供 stream（优先复用库）。

## Decisions

### D1: Trait 默认 fallback，而非拆第二套 StreamingChatModel

- **选择**：在 `ChatModel` 增加 `complete_streaming(..., on_delta) -> ModelTurn`，默认实现调用 `complete` 再分块 + `Meta(Fallback)`。
- **理由**：测试/Scripted/缺口协议零改动即可满足 C；loop 单一路径。
- **备选**：独立 trait — 双份实现与 mock，成本高，否决。

### D2: 工具仍等完整 ModelTurn，不增量 dispatch

- **选择**：流式只推 text/reasoning；`ToolCalls` 在本轮结束后校验执行。
- **理由**：半截 JSON 执行不安全；与现有 harness 一致。
- **备选**：增量解析 tool_calls — 复杂且易误触发，首期不做。

### D3: 轨迹持久化用 JSON 列，保留扁平行字段

- **选择**：`trajectory_json` + 继续写 `content`/`reasoning`/`tool_calls_json`。
- **理由**：旧 UI/查询不破；轨迹保证多轮工具穿插顺序。
- **备选**：只存轨迹再派生正文 — 迁移与复制逻辑更绕，否决。

### D4: 前端时间线组件内嵌于助手回合

- **选择**：`AssistantTrajectory` 消费活跃轨迹与历史 `msg.trajectory`；去掉「消息外孤立 pendingTools 列表」为主路径。
- **理由**：对齐产品「一条回合一条故事」；折叠策略 1 易实现。

### D5: 内联 `<think>` 用流式状态机

- **选择**：增量剥离 open/close；未闭合前内容进 reasoning delta，标签不进正文。
- **理由**：本地小模型常见；与既有 `split_think_tags` 语义对齐并扩展到流式。

### D6: 后端落地优先级

1. Embedded：采样循环回调 delta → Live  
2. OpenAI Chat 兼容（含 Ollama `/v1`）：`stream:true` SSE → Live  
3. genai 路径：库 stream API 若可用 → Live，否则 Fallback  
4. 其余：默认 Fallback  

## Risks / Trade-offs

- [SSE/协议差异] → 解析失败时降级 Fallback 并打日志，不中断回合。  
- [流式中取消] → 统一 `cancel`；已 emit 步骤保留；Interrupted。  
- [trajectory 与扁平行不一致] → 写入时同一事务/同一函数同时写；以前端轨迹为准展示。  
- [假流式延迟体感] → fallback 角标降低预期；真流式优先覆盖本地与 Ollama。  
- [事件洪水] → 可按字符/时间合并 delta（实现细节），规格只要求增量可见。

## Migration Plan

1. Migration 增加 `trajectory_json`（默认空）。  
2. 读写 API 返回该字段；前端无字段时合成。  
3. 新回合写轨迹；旧数据只读合成。  
4. 回滚：忽略新列与未知 `stream_meta` 即可（旧前端可忽略）。

## Open Questions

- genai 各 Adapter 的 stream API 完备程度以实现期探测为准；不完备则该协议 Fallback（不改规格）。
