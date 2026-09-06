## Context

See proposal.md — Why。现状：`harness` 已有 SmallLocal/Default 策略、reask、卡住熔断、≥60 golden 与四率报告；Skill 为关键词 top-3；embedded 采样与 n_ctx 硬编码；`LlmProfile` 无生成参数；目录已有 `recommended` 但未区分 Agent/轻量文案。本设计跨 `agent/*`、`commands/llm*`、`model_catalog`、设置 UI 与 `skills/*.md`。

## Goals / Non-Goals

**Goals:**
- 用首错归因让 eval 能驱动修 harness
- Skill 路由文案 + 近邻降权，不换检索引擎
- 提示「工具目录常驻 + Skill 按需」；embedded 内按模型档位分强化强度
- profile 可配 temperature / top_p / max_tokens /（local）n_ctx，设置页可改，推理链路真正读取

**Non-Goals:**
- BM25/向量、多 Agent、长会话压缩、默认开 grammar、后训练

## Decisions

### D1. 首错类别枚举与归因时机
- **决策**：固定六类字符串枚举；逻辑层对 `model_output` + expect_* 做规则归因；e2e 在循环结束后对比 expect 与实际调用/最终文本再归类。
- **备选**：LLM-as-judge 归因 → 否决（本地小模型不可靠、CI 无权重）。
- **answer_mismatch**：仅在「工具结果已成功且 expect 有可比对要点」时检查最终助手文本是否包含关键片段；无要点则跳过该类。

### D2. 模型档位如何进 `strategy_for`
- **决策**：`model_catalog` 增加 `agent_tier: recommended | lite`（或复用/扩展 `recommended` + 展示 label）；`strategy_for(backend, model)` 对 embedded：recommended → 完整强化；lite → 强化但更短 few-shot / 更紧 budget；非 embedded → Default。
- **备选**：仅改文案不改策略 → 否决（用户明确要求档位与策略一起动）。

### D3. 生成参数挂在 profile 而非全局
- **决策**：字段进 `LlmProfile` / `LlmProfileInput` / TS 类型；`LlmConfig` 投影一并带上以便 `resolve_backend` 使用。默认值不写盘（`Option` + serde skip empty）。
- **范围校验**（保存时）：temperature ∈ [0, 2]；top_p ∈ (0, 1]；max_tokens ∈ [1, 128000]；n_ctx ∈ [512, 32768] 且建议 256 对齐（实现可放宽到 64 对齐）。
- **云端**：经 genai/`ChatOptions` 或请求体传 temperature/top_p/max_tokens；不支持的协议降级为仅传支持的字段并打日志，不得失败整轮。
- **embedded**：`build_sampler` 与 `LlamaContextParams::with_n_ctx` 读激活 profile；n_ctx 变更触发模型上下文重建（与现有加载路径一致）。

### D4. Skill 近邻降权
- **决策**：在 `score_skill` 增加轻量规则：若 query 命中某 Skill 的「反例/不用」关键词表（可来自 frontmatter 可选 `avoid_keywords` 或正文约定段落），对该 Skill 降权；优先保证已知混淆对（json.to_query ↔ json.flatten 等）单测覆盖。
- **备选**：立即上 BM25 → 留待后续 change。

### D5. 提示布局
- **决策**：`build_small_prompt` 固定顺序：角色 → 工具目录摘要 → 命中 Skill → few-shot；lite 档减少 few-shot 正例数量但仍保留负例；预算断言按档位常量分设。

## Risks / Trade-offs

- [归因规则过粗] → 先覆盖六类确定性规则，允许「unknown」桶计数但不进期望枚举强制断言。
- [n_ctx 改大导致 OOM] → UI 提示内存风险；默认仍 4096；保存上限 32768。
- [云端 API 忽略部分采样字段] → 文档化降级；以 local/embedded 为可测真源。
- [Skill 全文 + 目录仍可能触顶] → 档位预算 + 现有 char budget 测试。

## Migration Plan

- 旧 `profiles.json` 无新字段：反序列化默认 `None`，行为与现网一致。
- 目录 label 文案更新为含 Agent/轻量；`recommended` 语义保持「默认推荐下载」。
- 回滚：还原 profile 字段与 harness 策略即可，无 DB migration。

## Open Questions

- 是否在本期目录增加第三档更大模型（如 3B）：**不阻塞**；规格只要求档位区分与推荐≥1.5B，不加新权重也可只改文案与策略分支。
