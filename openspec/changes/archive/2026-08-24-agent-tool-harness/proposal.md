## Why

内置本地模型（Qwen2.5-0.5B/1.5B Instruct）在当前 Agent 循环中表现很差：系统提示只是把全部工具的 OpenAI JSON 一次性塞入，缺少意图引导与少样本示例；`<tool_call>` 输出全靠模型自由生成，小模型经常产出格式非法、参数错位或工具选错的调用；失败后没有修复重试，直接把错误抛给用户。需要一个面向小模型的 harness 工程，让所有后端（embedded / Ollama / 云端）统一接入，但按模型能力搭配不同策略，使内置 0.5B 模型也能稳定完成「理解意图 → 调对工具 → 给出预期输出」的闭环，并以离线评测集驱动持续迭代。

## What Changes

- 新增**可插拔 Harness 策略层**：按当前后端/模型选择策略档位（如 `small-local` / `large`），所有后端统一走该层；策略决定提示模板、是否启用约束解码、修复重试预算等。
- **提示工程强化**：为小模型提供结构化 system prompt（任务说明 + 工具中文说明书 + 少样本 tool_call 示例 + 意图-工具映射引导），替换现有一次性 JSON 倾倒；检索到的 Skill 注入方式保持。
- **约束解码**：嵌入式引擎（llama-cpp-2）在需要工具调用的回合支持 GBNF/grammar 约束采样，保证 `<tool_call>` 块 JSON 结构合法（工具名 ∈ 注册表、参数结构符合 schema 的形状约束）。
- **输出解析与修复重试**：解析层容错（多余空白/代码围栏/中英文引号等）；参数经 JSON Schema 校验，不合格时把校验错误回填给模型并在预算内重试（reask），而非直接失败。
- **离线评测集与评测命令**：内置 golden cases（用户输入 → 预期工具与参数 → 预期输出要点），提供一条 Rust 侧评测入口（test 或 `cargo xtask`-style 命令），报告意图命中率 / 工具选择正确率 / 参数合法率 / 端到端成功率，作为 harness 迭代的回归门槛。
- 前端无新页面；Agent 对话流式事件结构不变（ToolStart/ToolEnd 复用），仅可能新增修复重试的可见标注（复用现有事件字段，不破坏契约）。

### Non-goals

- 不为 Harness 注册任何新的可执行工具；可调用工具集合仍仅来自现有注册表（side_effect=none 纯计算工具）。
- 不改变本地 LLM 运行时生命周期（下载、加载、回退到 Ollama 等行为见 local-llm-runtime，不变）。
- 不做云端模型微调 / 不下载额外权重。
- 不引入第三方 Skill 安装机制。
- 评测集为开发者回归工具，不在用户 UI 中做评测面板。

## Capabilities

### New Capabilities
- `agent-tool-harness`: 面向小模型的统一 Agent harness：可插拔策略层、强化提示模板、约束解码、解析容错与修复重试、离线评测集与评测命令。

### Modified Capabilities
- `agent-chat`: Agent Tool Loop 的系统提示构建与工具调用回合行为由 harness 策略层接管（提示构建、解析校验、重试语义在 spec 层收紧：失败 MUST 回填重试而非直接终止）。

## Impact

- **Rust**：`src-tauri/src/agent/loop.rs`（接入策略层、修复重试）、`embedded_engine.rs`（grammar 约束采样支持）、新增 `src-tauri/src/agent/harness/`（策略、提示模板、解析修复、schema 校验）与评测模块（如 `src-tauri/tests/` 或 `src-tauri/src/agent/harness/eval.rs` + 命令）。
- **前端**：`src/views/`（AgentChat 相关视图）仅在需要展示「重试中」标注时微调；无路由/工具注册变更。
- **Rust command**：无新增用户可见 command；评测入口走测试或 cargo 命令。
- **工具 id**：无新增。
- **依赖**：可能新增 JSON Schema 校验 crate（如 `jsonschema`）；`llama-cpp-2` 的 grammar/sampler 能力需确认版本支持。
- **规格**：`openspec/specs/agent-tool-harness/` 新建；`openspec/specs/agent-chat/spec.md` 修订。
