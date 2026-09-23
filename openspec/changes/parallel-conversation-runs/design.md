## Context

今日 `AgentCancel` 为全局单一 `AtomicBool`，`HomePage` 丢弃非 `activeId` 的 `agent-event`，侧栏无运行指示。Superpowers 设计见 `docs/superpowers/specs/2026-09-22-sidebar-run-indicators-design.md`（方案 B：进程内运行注册表）。实现计划见 `docs/superpowers/plans/2026-09-22-parallel-conversation-runs.md`。

## Goals / Non-Goals

**Goals:**

- 多会话并行 Agent 回合（每会话至多一轮）
- 按会话取消与 `agent-run-status` 事件
- 侧栏空闲/运行两态图标
- 切走继续、切回接上 live 缓冲

**Non-Goals:**

- 侧栏取消、失败/未读角标、SQLite run 表、全局并发上限、同会话排队

## Decisions

1. **进程内 `RunRegistry`（非仅前端 busy map）**  
   - 理由：真并行需要按会话 cancel；注册表是单一真相。  
   - 备选：仅前端并行伪装（取消会误伤）— 否决。

2. **管理 `Arc<RunRegistry>`**  
   - 理由：`spawn_blocking` worker 需在收尾时 `finish`，不能只靠主线程 `State`。  
   - `try_begin` 返回该会话的 `Arc<AtomicBool>` 给 `run_agent`。

3. **事件名 `agent-run-status`**  
   - payload：`{ conversationId, status: "running"|"idle" }`（serde camelCase）。  
   - 与流式 `agent-event` 分离，侧栏可只订状态。

4. **前端 `agentRuns` store**  
   - `runStatusById` + `liveById`；`HomePage` 不再丢弃非 active 事件。  
   - `done`/`interrupted`：清 live；若为 active 则 `openConversation` 从 DB 重载。

5. **`cancel_chat_turn(conversationId)` BREAKING**  
   - 无参全局取消删除；未知/idle → `Ok(())`。

6. **删除 running：先 cancel 再删**  
   - 避免 worker 向已删会话继续写状态。

## Risks / Trade-offs

- [刷新丢 live 缓冲] → 与现状一致；落库后重开靠 DB。  
- [双 LLM 并行压垮本机] → 本版不设全局上限；后续可加。  
- [BREAKING cancel 签名] → 同步改唯一前端调用点。

## Migration Plan

- 前端与 Rust 同版本发布；无 DB migration。  
- 回滚：恢复全局 `AgentCancel` 与旧 cancel 签名（丢失并行能力）。

## Open Questions

（无 — 已在设计中拍板）
