# Brainstorm: llm-multi-provider-protocols

> Raw capture of superpowers:brainstorming（OpenSpec 要求写入本 change，不写 `docs/superpowers/specs/`）。
> 日期：2026-08-22

## Background

TBox 已有 `local`（llama.cpp sidecar + 精选 GGUF）、OpenAI / DeepSeek / Anthropic / custom 配置；Agent 侧几乎只走自写 OpenAI Chat Completions（`reqwest`）。Anthropic 对话实际不可用；GGUF 下载后端已有进度事件，前端缺进度条与成功提示。用户要求：下载进度与成败提示、多提供商（含 Ollama）、协议可选、优先用现成 Rust LLM 库，并与最新 CC Switch 提供商列表对齐。

## Decision chain

### Q1 — 第一期提供商范围
- 初版候选：Ollama + 打通 Anthropic + 现有
- **用户追加**：提供商列表对齐 **最新版 CC Switch** Claude 预设（全量，约 77 项）
- **决议**：以 [farion1231/cc-switch](https://github.com/farion1231/cc-switch) `main` 的 Claude provider presets 为预设真相来源；另加 TBox 自有 `local`、`ollama`

### Q2 — 协议是否与提供商解耦
- **决议：2A** — 提供商与协议解耦；用户可选协议并覆盖预设默认

### Q3 — local vs Ollama
- **决议：3A** — 并存；保留内置 GGUF + sidecar；Ollama 为独立提供商

### Q4 — 下载 UX
- **决议**：GGUF 用 ProgressBar + 成功/失败提示；Ollama pull 也要进度与成败提示

### Q5 — Responses API
- **决议**：第一期协议包含 `openai_responses`

### Q6 — 客户端路线
- A `genai` / B 自写 HTTP / C `multi-llm`
- **决议：A** — 引入 [rust-genai](https://github.com/jeremychone/rust-genai)

### Q7 — OAuth 预设（Copilot / Codex / xAI）
- **决议**：列表可见，标「需 OAuth，暂不支持」；不做 OAuth 登录

### Q8 — 设计分段审批
- §1 配置模型 → 通过（后经 CC Switch / Responses 修订再确认）
- §2 genai + Agent → 通过（后经提供商列表修订再确认）
- §3 下载/pull UX → 通过
- §4 风险与非目标 → 通过

## Approaches considered

| 路线 | 摘要 | 结论 |
|------|------|------|
| A `genai` | 统一多协议原生客户端 | **采用** |
| B 自写 reqwest 适配器 | 依赖少、维护协议成本高 | 否 |
| C `multi-llm` | 提供商面窄 | 否 |

## Validated design

### §1 配置模型（提供商 × 协议）

- `LlmConfig` 扩展：`provider`（预设 id 或 `local` / `ollama` / `custom` 等）、`protocol`、`base_url`、`model`、`has_api_key`
- **协议枚举（第一期）**：
  - `openai_chat` — OpenAI Chat Completions
  - `openai_responses` — OpenAI Responses API
  - `anthropic_messages` — Anthropic Messages API
  - `gemini_native` — Gemini native（对齐 CC Switch `gemini_native`）
  - `ollama_native` — Ollama 原生
- 预设来自 CC Switch Claude 全量快照（注明上游版本/日期）；选预设填充默认 base_url / model / protocol，用户可改协议
- 旧配置无 `protocol`：按提供商/预设默认推断，不强制改写已有选择
- API Key 仍只存 Rust 侧加密文件

### §2 `genai` 与 Agent

- 保留 `ChatModel` / `ModelMessage` / `ToolCall` / `ModelTurn`
- 新增基于 `genai` 的适配器，按 protocol 选 adapter + 自定义 endpoint/auth
- `local`：继续 sidecar `127.0.0.1:11435`，协议 `openai_chat`；不占用 Ollama `11434`
- `resolve_backend`：打通 Anthropic / Ollama / Responses / Gemini native；缺配置时明确失败，禁止静默打云端
- `test_llm_connection` 按协议探测
- 第一期不做：删光全部自写 reqwest；不做 CC Switch 本地代理/热切换/故障转移

### §3 下载 / Pull UX

- GGUF：ProgressBar + 成功 toast/行内成功 + 失败可读错误；半截不可启用
- Ollama pull：进度事件 + 成败提示；失败不标可用
- 可取消（协议支持时）；不阻塞设置页其它操作

### §4 风险与非目标

**风险**：genai tool-calling 对齐；预设漂移与推广链；默认协议与真实端点不符；OAuth 误解；下载失败体验  
**缓解**：契约测试；预设快照+日期；协议可改+连接测试；OAuth 明确不可用；进度+成败+不装半截  

**非目标**：CC Switch 代理/热切换/failover；OAuth；用 Ollama 替换 local sidecar；Spotlight 绑对话 LLM；完整删除自写 HTTP 客户端

## Recommended next (OpenSpec)

继续本 change 的 proposal → specs → design → tasks → plan；实现阶段再 `/openspec-apply-change`。
