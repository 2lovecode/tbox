## Context

TBox 已有 `user_memories` / revisions、写时 `decide_action`、`retrieve_for_inject` 与设置面板。缺口是抽取：`extract_and_ingest_session_delta` 只调用 `extract_candidates_heuristic`，与 living spec「MUST 使用当前可用 LLM」不符。注入路径在 `loop.rs` 用同一 `auto_memory_enabled` 开关包住，导致关自动记忆后也不注入。

约束：本地/云端 LLM 经既有 `ChatModel`；抽取不得拖垮主对话；密钥仍由 `looks_like_secret` 拦截。

## Goals / Non-Goals

**Goals:**

- 回合结束用当前配置 LLM 抽取结构化候选，再 `ingest_candidates`
- LLM 失败 / 空结果 / 解析失败 → 启发式回退
- `auto_memory_enabled=false` 只停抽取，不停注入

**Non-Goals:**

- Agent 状态栏、向量检索、记忆压缩、改 system 注入位置（仍可短 system 段）
- 子 Agent / 多模型专用抽取配置 UI

## Decisions

1. **抽取调用形态**  
   - 选用：同步、短 prompt、无工具、非流式的一次 `complete`（或等价），输入为「本回合 user + 最终 assistant 文本」（已有 `delta`），输出 JSON 数组 `{key?, text}`。  
   - 备选：后台线程异步抽取——更不挡 UI，但时序与测试更复杂；本变更优先同步且超时/失败即回退，保持 loop 简单。  
   - 理由：与「回合结束」触发一致；失败非致命。

2. **Prompt 契约**  
   - 明确：只抽稳定偏好/身份/约束；禁止工具原文、密钥、一次性行程细节；最多 N 条（建议 ≤8）。  
   - 解析：宽松 JSON（允许 markdown fence）；非法则启发式。

3. **开关语义**  
   - `auto_memory_enabled`：仅门控 `extract_and_ingest_*`。  
   - `retrieve_for_inject`：只要有生效记忆即注入（仍受预算）。

4. **启发式保留**  
   - 作为离线/小模型/失败兜底，不删除；单测继续覆盖规则路径。

## Risks / Trade-offs

- [额外 LLM 费用/延迟] → 仅回合结束一次；短输出；失败立即回退  
- [幻觉写入错误偏好] → 写时消歧 + 用户可删/撤销；prompt 强调「仅用户明确陈述」  
- [密钥漏进候选] → 入库前 `looks_like_secret` 仍强制 NOOP  
- [同步抽取拖慢 Done] → 可后续改为 spawn；本版接受短延迟，并在 design 留迁移点

## Migration Plan

- 无 schema 迁移。部署后新回合自动走 LLM 抽取；旧记忆不变。  
- 回滚：保留启发式入口，可临时强制只走 heuristic（实现可用 feature 或设置扩展；非本版必做 UI）。

## Open Questions

- 是否在设置增加「仅启发式抽取」调试开关：本版不做，需要时再加。
