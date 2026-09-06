## Context

See proposal.md — Why。当前 Agent 循环已有 harness、轨迹持久化（`trajectory_json`）与 profile 级 `n_ctx` / `max_tokens`，但缺少「发往模型的真实占用」观测、会话内 Runtime 压缩，以及跨会话用户记忆。本设计覆盖三层数据与一条写路径，实现须对齐 delta specs，不在此重复需求条文。

## Goals / Non-Goals

**Goals:**
- 单一 `ContextBudget` 源驱动 UI、阈值压缩与 memory 注入上限
- Runtime 可压缩；Audit 轨迹 append-only 可回源
- 会话结束（缓冲）自动抽取；写入时 ADD/UPDATE/DELETE/NOOP，UPDATE 带 revisions
- 设置页可管理记忆；敏感候选硬拒绝

**Non-Goals:**
- 子 Agent 隔离、双模型审核 PR、向量库级共享 RAG、每条消息实时抽取（见 proposal Non-goals）

## Decisions

1. **三层存储分离**  
   - Audit：现有会话消息 + 轨迹（UI/历史）  
   - Runtime：本轮实际 `messages`（可含摘要替换、压缩标记）  
   - Memory：独立 SQLite 表（生效行 + revisions）  
   备选：在消息上原地改写 — 否决，会破坏时间线与回看。

2. **Token 估算**  
   首版用确定性近似（字符启发式或后端 tokenizer 若易得）；与压缩/UI 同源。不要求首版字节级精确对齐各提供商计费。  
   上限：local → `n_ctx`；云端 → 可配置/提供商默认常量。

3. **压缩分层落地顺序**  
   L1 工具结果截断/落盘摘要必做；阈值批量替换旧 tool results；全量 LLM 摘要 + 熔断为二期可开关能力，tasks 中靠后。  
   触发默认 `ratio >= 0.8`，可配置。禁止每轮全量压。

4. **记忆写入：Mem0 v2 风格 UPDATE**  
   Extract（一次 LLM）→ 相近检索（条目少可全表/关键词；后续可加嵌入）→ Decide ADD/UPDATE/DELETE/NOOP → 闸门（密钥正则等）→ 落库。  
   UPDATE 前写 revisions，支持设置页撤销。不做完整 Proposer/Reviewer。

5. **触发与注入**  
   抽取：会话结束/切换或「回合结束且有增量」，异步、失败非致命。  
   注入：构建 prompt 时 top-k（默认 ≤8），硬上限 min(5% 窗口, 固定字符)；计入 budget.`memory`。  
   开关：`auto_memory_enabled`（默认开）。

6. **事件与 UI**  
   扩展 `agent-event`（或等价）：`context_budget` 快照；可选 `compress` 提示。聊天输入区附近展示用量；设置增加「记忆」管理块。

## Risks / Trade-offs

- [错误 UPDATE 丢偏好] → revisions + UI 撤销；敏感类拒绝入库  
- [估算与真实 token 偏差] → 阈值略保守；文档标明「估算」  
- [抽取额外 LLM 成本/延迟] → 异步缓冲；小模型可关自动抽取  
- [压缩丢失细节] → Audit 留原文；摘要带 source_ref  
- [压缩死循环烧调用] → 熔断计数  

## Migration Plan

- SQLite：新增 memory / memory_revisions 表（或等价），向后兼容 migration  
- 旧会话无预算事件：UI 隐藏或显示「—」直至首轮新请求  
- 回滚：关闭自动记忆与压缩 feature flag（若实现）即可降级为旧行为；表可保留

## Open Questions

- 云端默认上下文窗口常量表：实现期按主流提供商填一版可覆盖默认即可  
- 首版是否暴露「手动压缩」按钮：非必须，可仅自动阈值触发
