## Context

TBox（Tauri 2 + Vue 3 + Rust）在设置中配置 LLM，Agent 循环在 `src-tauri/src/agent/`。当前状态：

- 提供者：`local` | `openai` | `deepseek` | `anthropic` | `custom`
- Agent 主路径：自写 OpenAI Chat Completions（`reqwest`）；`Anthropic` 在 `resolve_backend` 直接 `LlmUnavailable`
- 本地推理：llama.cpp sidecar（`127.0.0.1:11435`），精选 GGUF 下载已有进度/失败事件，UI 缺 ProgressBar 与成功提示
- 密钥：`~/.toolbox/llm_config.json` + 加密 `llm_secret.bin`

用户已确认：引入 `genai`；提供商对齐 CC Switch 最新 Claude 全量预设；协议与提供商解耦（含 Responses / Gemini native）；保留 local 并加 Ollama；GGUF 与 Ollama pull 均需进度与成败提示。

## Goals / Non-Goals

**Goals:**

- 设置页可从 CC Switch 对齐的预设（+ local / ollama）选择提供商，并独立选择协议。
- Agent 与连接测试按协议经 `genai` 工作（配置完整时）。
- GGUF 下载与 Ollama pull 展示进度条，成功/失败明确提示，失败不标已安装。
- 旧配置可迁移推断 `protocol`，不强制改写用户已选提供商。

**Non-Goals:**

- CC Switch 本地代理 / 热切换 / failover
- OAuth 提供商实际登录
- 移除 local sidecar 或改用 Ollama 替代
- 删除全部遗留 `reqwest` 客户端代码
- Spotlight 改绑对话 LLM
- 自动同步上游预设每一次变更

## Decisions

### D1：客户端库选 `genai`
- **选择**：`genai` 作为 Agent 与连接测试主路径。
- **理由**：原生覆盖 OpenAI Chat / Responses、Anthropic、Ollama、Gemini 等；符合「用现成 Rust LLM 库」。
- **已考虑 alternative**：继续扩自写 `reqwest`（维护成本高）；`multi-llm`（面窄、流式不成熟）。

### D2：提供商 = CC Switch Claude 预设快照 + local + ollama
- **选择**：仓库内静态预设表（id、显示名、默认 base_url、默认 model、默认 protocol、是否 oauth）；定期人工对照上游。
- **理由**：用户明确要求最新版 CC Switch 列表；桌面应用需离线可读元数据。
- **已考虑 alternative**：运行时拉远程列表（依赖网络、隐私）；仅常用 30 项子集（用户确认全量）。

### D3：协议与提供商解耦
- **选择**：`protocol` ∈ `openai_chat` | `openai_responses` | `anthropic_messages` | `gemini_native` | `ollama_native`；换预设填默认，可覆盖。
- **理由**：中转站常「挂 Anthropic 皮走 OpenAI 骨」或相反；用户需手动纠正。
- **已考虑 alternative**：协议完全由提供商锁定（不够灵活）。

### D4：local 与 Ollama 并存
- **选择**：`local` 继续 sidecar + GGUF；`ollama` 独立，默认 `http://127.0.0.1:11434`、`ollama_native`。
- **理由**：已交付的离线路径不能破坏；Ollama 用户另有工作流。
- **已考虑 alternative**：用 Ollama 替换 sidecar（破坏现有安装与端口约定）。

### D5：OAuth 预设仅展示
- **选择**：Copilot / Codex / xAI 等 `requiresOAuth` 预设出现在列表，但禁用启用/保存为可用后端，文案标明暂不支持。
- **理由**：全量列表对齐的同时控制第一期范围。
- **已考虑 alternative**：第一期隐藏 OAuth 项（与「全量列表」冲突）。

### D6：下载 UX
- **选择**：前端 `ProgressBar` + 成功/失败提示；Ollama pull 独立进度事件，组件形态复用。
- **理由**：后端进度事件已有，补齐可观察行为。
- **已考虑 alternative**：仅 toast 百分比（不如条形进度直观）。

### D7：保留 ChatModel 边界
- **选择**：`genai` 包在适配器后，仍实现现有 `ChatModel`，Agent 循环与前端流式事件契约不变。
- **理由**：降低回归面；工具循环已依赖 `ModelTurn`。

## Risks / Trade-offs

- [Risk] `genai` tool-calling 与现有 OpenAI tools JSON 不完全对齐 → Mitigation: 适配层契约测试；必要时单协议降级提示。
- [Risk] 预设含推广 affiliate URL、上游频繁变更 → Mitigation: 快照 + 注明 commit/日期；不自动同步。
- [Risk] 预设默认 protocol 与真实端点不符 → Mitigation: 协议可改 + 连接测试。
- [Risk] 用户误以为 OAuth 可用 → Mitigation: UI 禁用态 + 明确文案。
- [Trade-off] 全量 ~77 预设使下拉变长 → 接受理由：用户要求对齐 CC Switch；可用搜索/分组后续再优化（本 change 可用简单分组或可搜索 select 若成本低）。
- [Trade-off] 保留遗留 reqwest 代码一段时间 → 接受理由：降低一次性删除风险。

## Migration Plan

1. 扩展 `LlmConfig` 增加 `protocol`（serde 默认/推断兼容旧 JSON）。
2. 发布预设表与设置 UI；旧用户打开设置时看到推断协议，不改其 provider。
3. Agent 切换到 `genai` 适配器；回归：local / OpenAI 兼容 / Anthropic / Ollama（有环境时）。
4. 下载 UI 与 Ollama pull 事件联调。
5. Rollback：保留旧 `OpenAiCompatModel` 编译路径或 feature 开关（实现期定）；配置多字段向后兼容读取。

验收：设置可选协议与预设；GGUF/Ollama pull 有进度与成败提示；配置完整时 Anthropic/Ollama 对话不再恒失败；无模型 local 不静默打云端。

## Open Questions

- 设置页预设下拉是否第一期就做搜索过滤（建议：若选项 > 30 则做简单 filter，实现 tasks 中确认）。
- `genai` 具体 crate 版本以 `cargo add` 时 crates.io 稳定版为准，锁定在 Cargo.lock。
