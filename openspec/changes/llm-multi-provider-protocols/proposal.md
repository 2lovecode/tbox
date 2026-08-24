## Why

设置页虽能配 LLM，但 Agent 几乎只走自写 OpenAI Chat Completions：Anthropic 不可用、无 Ollama、协议不可选；精选 GGUF 下载仅有字节文案，缺少进度条与成败提示。现在补齐多提供商（对齐 CC Switch 最新 Claude 预设）、多协议与下载/pull UX，并引入现成 Rust 库 `genai`，才能让对话设置真正可用、可扩展。

## What Changes

**模型下载 UX**
- From: 下载中仅显示已收/总量字节；失败有部分提示，成功无明确反馈。
- To: ProgressBar 百分比进度；成功与失败均有明确提示；半截文件不可启用。
- Reason: 规格已要求进度，UI 未兑现。
- Impact: non-breaking；设置页 local 模型区。

**Ollama 提供商 + pull**
- From: 无 Ollama；本地仅 sidecar + GGUF。
- To: 独立 `ollama` 提供商；可 pull 模型并显示进度与成败。
- Reason: 用户常用本机 Ollama，与内置 local 并存。
- Impact: non-breaking；新增配置与命令。

**多提供商预设（CC Switch 对齐）**
- From: local / openai / deepseek / anthropic / custom。
- To: CC Switch Claude 全量预设快照 + `local` + `ollama`；OAuth 类仅展示且标明暂不支持。
- Reason: 降低国产/中转端点配置成本。
- Impact: non-breaking；配置枚举/元数据扩展。

**协议可选**
- From: 隐式 OpenAI Chat Completions（Anthropic 路径失败）。
- To: 用户可选 `openai_chat` / `openai_responses` / `anthropic_messages` / `gemini_native` / `ollama_native`；预设带默认，可覆盖。
- Reason: 不同端点原生协议不同。
- Impact: non-breaking；旧配置推断 protocol。

**Agent 客户端**
- From: 自写 `reqwest` OpenAI 兼容客户端；Anthropic 不可用。
- To: 主路径经 `genai` 适配到现有 `ChatModel`；按协议发请求（含 tool calling）。
- Reason: 复用现成 Rust 多协议库。
- Impact: non-breaking 对前端事件；Rust Agent 内部替换。

## Capabilities

### New Capabilities

- （无）本 change 不新增 capability 目录名；行为落在既有 `local-llm-runtime` / `agent-chat`。

### Modified Capabilities

- `local-llm-runtime`: 多提供商预设、协议字段、Ollama、下载/pull 进度与成败提示、基于 `genai` 的连接测试。
- `agent-chat`: Agent 按当前 protocol/provider 调用 LLM；Anthropic / Ollama / Responses / Gemini native 可用（在配置完整时）；仍禁止无模型时静默打云端。

## Impact

**前端**

- `src/components/settings/SettingsModal.vue`：ProgressBar、成功/失败提示、提供商预设列表、协议下拉、Ollama pull UI。
- `src/types/llm.ts`、`src/stores/llm.ts`：provider / protocol 类型与元数据。

**Rust**

- `src-tauri/src/commands/llm.rs`：配置结构、预设表、`test_llm_connection`。
- `src-tauri/src/commands/model_catalog.rs`：下载成功事件（若缺）与前端对齐。
- 新增 Ollama pull 相关 command + 进度事件。
- `src-tauri/src/agent/llm.rs`：`genai` 适配器；`resolve_backend` 扩展。
- `src-tauri/Cargo.toml`：依赖 `genai`。

**其它**

- 仓库内维护 CC Switch Claude 预设快照（注明上游日期/commit）；不做代理/热切换/OAuth。
- 工具 id：无新工具页；设置入口仍为现有 Settings 模态。

## Non-goals

- CC Switch 本地代理、热切换、故障转移。
- OAuth 登录（GitHub Copilot / Codex / xAI 等）。
- 用 Ollama 替换或移除 local sidecar。
- 完整删除全部自写 `reqwest` LLM 代码。
- 改变 Spotlight / 各独立工具页行为。
- 自动跟随上游 CC Switch 每一次预设更新。
