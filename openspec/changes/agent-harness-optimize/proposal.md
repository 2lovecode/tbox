## Why

本地小模型 Agent 已有 harness（策略层、reask、eval 四率），但失败只能看汇总成功率，无法按根因修；Skill 检索与提示注入尚未按「路由条件」优化；采样参数硬编码，用户无法在设置里调 temperature / top_p / max_tokens / 本地上下文。顾问结论要求用评测归因驱动 Harness 迭代，并拉开 0.5B 与推荐档模型体验——现在动手可直接落在现有 capability 上增量改。

## What Changes

- **Eval 失败归因**：e2e / 逻辑层报告增加首错类别计数（intent / tool_choice / args / format / loop / answer_mismatch）；golden cases 支持可选预期类别；每类至少若干轨迹前缀回归用例。
- **Skill 路由文案**：16 个预置 Skill 的 keywords / 正文边界改为「何时用 / 何时不用 / 反例」；检索对「不用」近邻降权；仍限 top-3，不做 BM25/向量。
- **小模型提示结构**：工具一行摘要常驻；Skill 全文按检索注入；维持字符预算；`strategy_for` 按模型档位（轻量 0.5B vs Agent 推荐 ≥1.5B）区分强化强度。
- **模型目录与推荐文案**：目录标明「Agent 推荐」与「轻量（弱工具）」；推荐项保持 ≥1.5B；约束解码默认仍关闭。
- **生成参数可配置**：每个 LLM profile（含 local）可配置 temperature、top_p、max_tokens；local/embedded 另可配置 n_ctx（上下文长度）；设置页可编辑；Agent 循环与嵌入式采样链 MUST 使用激活 profile 的参数（缺省用内置默认）。
- **Non-goals**：不上 BM25/向量检索、多 Agent、长会话压缩、SFT/RL、默认开启约束解码、不新增带副作用的工具。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-tool-harness`：失败归因、按模型档位策略、提示「目录 + 按需 Skill」、纠正错误分类映射保持可测。
- `app-layer-eval-suite`：评测报告与 cases 契约扩展归因字段与前缀回归覆盖。
- `skill-content-enrichment`：Skill 路由条件与反例；检索降权行为。
- `local-llm-runtime`：目录 Agent 档位标注；嵌入式推理读取可配置采样/上下文参数。
- `llm-provider-profiles`：profile 持久化与读写生成参数；设置 UI 契约。

## Impact

- Rust：`agent/harness/*`、`agent/skills.rs`、`agent/loop.rs`、`agent/embedded_engine.rs`、`agent/genai_model.rs`、`commands/llm.rs`、`commands/model_catalog.rs`
- 前端：`src/types/llm.ts`、`src/stores/llm.ts`、`src/components/settings/SettingsModal.vue`（及 LLM 相关设置子组件）
- 预置 Skill：`src-tauri/skills/*.md`
- 评测：`src-tauri/src/agent/harness/eval/cases.json`
- 验证：相关 `cargo test`、逻辑层 eval、有权重时 e2e；前端 `vue-tsc`
