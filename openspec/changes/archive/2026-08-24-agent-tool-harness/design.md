## Context

当前 Agent 循环（`src-tauri/src/agent/loop.rs`）对本地小模型只有一个极简 system prompt（"Prefer registered pure-compute tools..."），`EmbeddedChatModel::complete` 把注册表全部工具的 OpenAI JSON（`render_tools_prompt`）追加到 system 消息，靠模型自由输出 Qwen `<tool_call>` 文本协议；解析失败降级为纯文本，参数不做 schema 校验，dispatch 失败只回填一次错误，模型下一回合若仍失败就循环耗尽（MAX_TOOL_ITERATIONS=8）或直接吐错。

内置 0.5B 模型在这种条件下几乎无法稳定完成工具调用：意图-工具映射靠模型记忆、格式合法性靠运气、失败无修复路径。本设计引入统一的 harness 层，所有后端（embedded / Ollama / 云端 genai）都经过它，但策略可按后端/模型插拔。

## Goals / Non-Goals

**Goals:**
- 所有后端统一接入 harness 策略层；小模型（embedded 0.5B/1.5B）启用强化策略，大模型可用轻量策略。
- 小模型在 golden 评测集上的端到端成功率显著可度量（评测命令输出指标）。
- `<tool_call>` 结构合法性由约束解码（embedded）+ 解析容错 + schema 校验 + reask 重试多层保证。
- 评测集成为仓库内回归资产，`cargo test` 可跑（不需网络、不需真实权重的部分必跑）。

**Non-Goals:**
- 不新增可执行工具、不改变工具注册表与 side_effect 约束。
- 不做模型微调、不下载新权重、不改 local-llm-runtime 生命周期。
- 不做用户可见的评测 UI。
- 云端/Ollama 后端不做 grammar 约束（HTTP 协议无法约束解码），只用提示+解析+reask。

## Decisions

### D1: Harness 作为独立模块 + trait 策略，而非散落在 loop.rs
新增 `src-tauri/src/agent/harness/mod.rs`：

```rust
pub trait HarnessStrategy {
    fn build_system_prompt(&self, user_text: &str, skills: &[SkillDoc], tools: &Value) -> String;
    fn parse_completion(&self, text: &str) -> ParsedTurn;          // 容错解析
    fn repair_budget(&self) -> usize;                               // reask 次数
    fn constrained(&self) -> bool;                                  // 是否请求 grammar
}
pub fn strategy_for(backend: &Backend, model: &str) -> Box<dyn HarnessStrategy>;
```

- `SmallLocalStrategy`（embedded 0.5B/1.5B）：少样本提示 + 中文工具说明 + 容错解析 + reask 预算 2 + constrained=true。
- `DefaultStrategy`（云端/Ollama 大模型）：接近现状的简洁提示 + 同一套容错解析（解析容错对所有后端免费受益）+ reask 预算 1。
- 备选：用配置文件描述策略。否决——策略与代码（提示模板、解析规则）强耦合，配置化只增加间接层；档位选择本身只有两个，代码内判定即可（按后端类型 + 模型名前缀）。

### D2: 提示模板：结构化 + 少样本，工具说明中文化
`SmallLocalStrategy::build_system_prompt` 产出四段式：
1. 角色与任务（「你是 TBox 工具助手，优先用工具完成计算类请求」）；
2. 检索到的 Skill 正文（保持现有 `retrieve_skills` 结果注入，不扩大注入量）；
3. 工具清单：每个工具一行 `名称：中文用途 | 参数：名(类型,必填)：说明`（由注册表元数据渲染，不倒原始 OpenAI JSON——0.5B 对原始 JSON schema 利用率极低）；
4. 少样本示例：2~3 个「用户问 → `<tool_call>` → 工具结果 → 简短回答」的完整对话样例（覆盖编码/解码与计算两类意图），以及「与工具无关时直接回答」的负例。

few-shot 示例放系统提示末尾（最靠近生成位置），并显式给出「只输出一个 `<tool_call>` 块、不要编造参数名」的约束句。备选：在消息序列里插 assistant/user 假回合演示。否决——污染会话历史与持久化边界，且 ChatML 模板下系统提示内嵌样例已足够。

### D3: 约束解码：GBNF 语法在 embedded 引擎回合级启用
`llama-cpp-2` 支持 grammar sampler（`LlamaSampler::grammar`）。做法：
- 由注册表在运行时生成 GBNF：工具名枚举 + `arguments` 为松散 JSON 对象（键值只约束为 JSON 值，**不**逐工具展开参数 schema——GBNF 全量展开维护成本高且易与 schema 漂移；参数正确性交给 D4 的 schema 校验 + reask）。
- 两阶段回合：模型先自由生成「意图文本」；当 harness 判定需要工具（或第一回合含畸变 `<tool_call>` 痕迹）时，下一回合以 `<tool_call>` 前缀引导 + grammar 采样强制合法块。简化实现：回合开始即用「grammar 约束 `<tool_call>` 或纯文本二选一」的宽松语法（`(text | tool_call-block)+`），避免两阶段状态机。
- `EngineCmd::Complete` 增加 `grammar: Option<String>`；仅 `SmallLocalStrategy` 传入。Ollama 原生 `format: json`/grammar 不在本期范围。
- 备选：不做 grammar，只靠解析容错 + reask。保留为降级路径：若 `llama-cpp-2` 当前版本 grammar API 不可用或性能不可接受（0.5B CPU 下 grammar 采样开销需实测），`constrained()` 返回 false，其余层照常工作——这保证 D3 是增强项而非阻塞项。

### D4: 解析容错 + schema 校验 + reask 修复环
`parse_completion` 在现有 `parse_qwen_tool_calls` 基础上增加容错：剥代码围栏（```json 等）、全角引号/冒号归一、截去 `<tool_call>` 前的多余引导语后再解析、容忍 `arguments` 为字符串化 JSON。
随后每个 call：
1. 工具名不在注册表 → 回填错误文本（含相近工具名建议，编辑距离 ≤2）；
2. `arguments` 用 `jsonschema` crate 按该工具 JSON Schema 校验；失败 → 错误信息回填，进入 reask：把「校验错误 + 正确参数说明」作为 user 消息追加，重新请求模型，预算内（small=2 次）循环；
3. reask 耗尽 → 按 agent-chat 现有语义回填最终错误并让模型给出文字解释，不崩溃。

reask 计数与 MAX_TOOL_ITERATIONS 合并核算（避免叠加放大循环上限）。

### D5: 评测集：仓库内 golden cases + 双档评测命令
- 数据：`src-tauri/src/agent/harness/eval/cases.json`（或 include_str 的 md/json），每条含 `user_input`、`expect_tool`（可空=闲聊负例）、`expect_args_contains`、`expect_output_contains`。首期 ≥30 条，覆盖 12 个已注册工具 + 闲聊/无关请求负例 + 中英文表达。
- 驱动两层：
  - **纯逻辑层（必跑，CI）**：对 `parse_completion`/schema 校验/容错归一直接喂数据断言（不加载模型）；
  - **端到端层（可选，需权重）**：`cargo test --features agent-eval` 或独立 bin，加载已安装 GGUF 跑完整 `run_agent_on`，输出四项指标：意图命中率（该调/不该调判对）、工具选择正确率、参数合法率、端到端成功率。权重未安装时 skip 并提示。
- 指标不设硬门槛进 CI（0.5B 成功率受机器影响），但端到端报告必须可复现生成，供 harness 迭代对比。

### D6: 对 loop.rs 与前端的接入点
- `run_agent_on` 开头由注入的后端描述（现 `ChatModel` 需暴露 `backend_desc()`，或由调用方传入）选策略；提示构建移入策略，`build_system_prompt` 退役为薄封装。
- 前端事件不变；reask 发生时在既有 `ToolStart`/`ToolEnd` 流里自然可见（错误结果 + 后续再次 ToolStart），前端可选地把连续同工具调用标注为「重试」，契约不破坏。

## Risks / Trade-offs

- [grammar 采样在 CPU 0.5B 上显著拖慢生成] → 实测后可一键降级为纯解析容错路径（D3 备选），spec 层不把 grammar 定为 MUST 生效项，定为「支持并默认启用，可降级」。
- [GBNF 由注册表运行时生成，与工具定义漂移] → 生成逻辑与 `tools_as_openai_json` 同源（同一注册表元数据），加纯逻辑单测锁一致性。
- [reask 增加小模型回合数，体验变慢] → reask 预算小（≤2）且与 MAX_TOOL_ITERATIONS 合并核算；评测集监控平均回合数。
- [few-shot 系统提示变长，0.5B 4096 ctx 吃紧] → 工具说明压缩为单行摘要 + 只注入检索 Skill 的工具详情；评测含 ctx 长度断言（渲染后 prompt token 数 < 2048 目标值，超限告警）。
- [`jsonschema` 新依赖体积/编译时间] → 仅在 agent harness 模块使用；若不可接受，退化为手写必填字段/类型校验（注册表 schema 形状简单）。

## Migration Plan

1. 新增 harness 模块 + 策略 + 解析容错（纯函数，先行，行为可测）。
2. loop.rs 接入策略层（云端/Ollama 行为等价迁移）。
3. embedded grammar 支持（可独立开关）。
4. 评测集与评测命令补齐，跑基线报告。
5. 回滚策略：harness 模块整体位于 `agent/harness/`，loop.rs 接入点为单一调用，回退即恢复旧 prompt 路径。

## Open Questions

- ~~`llama-cpp-2` 当前锁定版本的 grammar sampler API 形态~~ → 已 spike 通过：
  `LlamaSampler::grammar(model, gbnf, "root")` 可用。**但任务 4.3 实测 0.5B + grammar
  意图命中率从 63.6% 跌至 12.1%，决策为 SmallLocalStrategy 默认关闭约束解码**，
  详见 `notes/constrained-decoding-eval.md`。
- 评测端到端层放 `cargo test --features` 还是独立 workspace bin → 实现期定为
  `cargo test --lib --features agent-eval`，无独立 bin。
