# Multi-LLM Providers & Download UX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **OpenSpec:** 实现前对照 `openspec/changes/llm-multi-provider-protocols/` 下 `specs/`、`design.md`、`tasks.md`。计划落点为本文件（勿写入 `docs/superpowers/plans/`）。

**Goal:** 设置页支持 CC Switch 对齐的多提供商预设与可选协议（含 Ollama），Agent 经 `genai` 按协议对话；GGUF 下载与 Ollama pull 显示进度条并提示成败。

**Architecture:** 扩展 `LlmConfig`（`protocol` + 预设 id）；静态预设快照供 UI；`genai` 适配器实现现有 `ChatModel`；local 仍走 sidecar OpenAI 兼容端口；下载/pull 经 Tauri 事件驱动 ProgressBar。

**Tech Stack:** Tauri 2、Vue 3、Rust、`genai`、现有 `reqwest`（遗留可暂留）、Ollama HTTP API。

## Global Constraints

- 行为必须满足本 change 的 `specs/local-llm-runtime/spec.md` 与 `specs/agent-chat/spec.md`
- API Key 只存 Rust 侧；前端仅 `has_api_key: bool`
- local sidecar 仅 `127.0.0.1`，不得占用 Ollama `11434`
- 无模型 / OAuth / 缺配置时禁止静默打云端
- 不做 CC Switch 代理、热切换、failover、OAuth 登录
- 提交信息与代码注释保持仓库既有风格；用户未要求则不要擅自 `git commit`（计划中的 Commit 步骤仅在 apply 且用户授权提交时执行）

## File map

| 路径 | 职责 |
|------|------|
| `src-tauri/src/commands/llm.rs` | 配置、protocol、连接测试 |
| `src-tauri/src/commands/llm_presets.rs`（新建） | CC Switch 预设快照 |
| `src-tauri/src/commands/ollama_pull.rs`（新建） | Ollama pull + 进度事件 |
| `src-tauri/src/commands/model_catalog.rs` | GGUF 下载；必要时补成功事件 |
| `src-tauri/src/agent/llm.rs` | `resolve_backend` + `GenaiChatModel` |
| `src-tauri/Cargo.toml` | `genai` 依赖 |
| `src/types/llm.ts` / `src/stores/llm.ts` | 前端类型 |
| `src/components/settings/SettingsModal.vue` | 预设、协议、ProgressBar、pull UI |
| `src/components/ProgressBar.vue` | 复用 |

---

### Task 1: Protocol on LlmConfig (TDD)

**Files:**
- Modify: `src-tauri/src/commands/llm.rs`
- Test: 同文件 `#[cfg(test)]`

**Interfaces:**
- Produces: `enum LlmProtocol { OpenaiChat, OpenaiResponses, AnthropicMessages, GeminiNative, OllamaNative }`；`LlmConfig.protocol`；`fn infer_protocol(provider) -> LlmProtocol`

- [ ] **Step 1: Write failing test** — 旧 JSON 无 `protocol` 反序列化后 OpenAI → `OpenaiChat`

```rust
#[test]
fn legacy_openai_config_infers_openai_chat() {
    let raw = r#"{"provider":"openai","base_url":"https://api.openai.com/v1","model":"gpt-4o-mini","has_api_key":true}"#;
    let cfg: LlmConfig = serde_json::from_str(raw).unwrap();
    assert_eq!(cfg.protocol, LlmProtocol::OpenaiChat);
}
```

- [ ] **Step 2: Run test — expect FAIL**（缺字段/类型）

Run: `cargo test -p toolbox legacy_openai_config_infers_openai_chat -- --nocapture`

- [ ] **Step 3: Add `LlmProtocol` + `#[serde(default)]` / custom default on `LlmConfig`**，按 provider 推断

- [ ] **Step 4: Run test — expect PASS**

- [ ] **Step 5: Commit**（仅当用户要求提交时）`feat(llm): add protocol field with legacy infer`

---

### Task 2: CC Switch preset snapshot module

**Files:**
- Create: `src-tauri/src/commands/llm_presets.rs`
- Modify: `src-tauri/src/commands/mod.rs`、`llm.rs`（list command）、`lib.rs` register
- Create/Update: 注释写明上游 `farion1231/cc-switch` commit SHA 与日期

**Interfaces:**
- Produces: `struct LlmPreset { id, label, default_base_url, default_model, default_protocol, requires_oauth, docs_url }`；`fn all_presets() -> Vec<LlmPreset>`；Tauri `list_llm_presets`
- 必须包含 id=`local`、`ollama`；OAuth 项 `requires_oauth=true`

- [ ] **Step 1: 从上游 `claudeProviderPresets.ts` 抽取为 Rust 静态表**（可用脚本生成一次，提交生成结果）
- [ ] **Step 2: 单测** `all_presets` 含 `local`/`ollama`，且至少 70+ 项；抽样 DeepSeek / Zhipu 默认 URL 非空
- [ ] **Step 3: `cargo test -p toolbox presets` 通过**
- [ ] **Step 4: Commit**（若授权）`feat(llm): add CC Switch-aligned provider presets`

---

### Task 3: Frontend types + settings provider/protocol UI

**Files:**
- Modify: `src/types/llm.ts`、`src/stores/llm.ts`、`src/components/settings/SettingsModal.vue`

**Interfaces:**
- Consumes: `list_llm_presets`、`get_llm_config` / `save_llm_config`（含 `protocol`）
- Produces: 协议下拉；OAuth option `disabled` + hint

- [ ] **Step 1: 扩展 TS 类型** `LlmProtocolId`、`LlmPresetMeta`
- [ ] **Step 2: Settings 加载预设；选预设填默认；协议可改；OAuth 不可保存为可用**
- [ ] **Step 3: 提供商 >30 时加 filter input**
- [ ] **Step 4: `npx vue-tsc --noEmit`**
- [ ] **Step 5: 手动**：选 DeepSeek → 默认协议；改 `anthropic_messages` 可保存

---

### Task 4: GGUF ProgressBar + success/failure feedback

**Files:**
- Modify: `src/components/settings/SettingsModal.vue`
- Modify（如缺成功事件）: `src-tauri/src/commands/model_catalog.rs` 增加 `model-download-succeeded` 或前端在 progress 达 total + 刷新列表后 toast

- [ ] **Step 1: 下载中渲染 `<ProgressBar :progress="pct" />`**，pct = `total>0 ? received/total*100 : indeterminate 处理`
- [ ] **Step 2: 监听失败事件已有 → 确保 `downloadError` / toast；成功后 `ElMessage`/行内成功 + `list` 刷新**
- [ ] **Step 3: 手动或事件注入验证** 进度条可见；取消后非已安装

---

### Task 5: Ollama pull command + UI

**Files:**
- Create: `src-tauri/src/commands/ollama_pull.rs`
- Modify: `lib.rs` register；`SettingsModal.vue`

**Interfaces:**
- Events: `ollama-pull-progress { name, completed, total, status }`、`ollama-pull-failed`、`ollama-pull-succeeded`
- Commands: `start_ollama_pull(name, base_url?)`、`cancel_ollama_pull(name)`

- [ ] **Step 1: 纯函数测试** 解析 Ollama pull NDJSON 一行 → 进度结构
- [ ] **Step 2: 实现 POST `/api/pull` stream + emit**
- [ ] **Step 3: UI** 模型名、Pull、ProgressBar、成败提示
- [ ] **Step 4: Ollama 未启动时失败提示清晰**

---

### Task 6: genai ChatModel adapter + resolve_backend

**Files:**
- Modify: `src-tauri/Cargo.toml`、`src-tauri/src/agent/llm.rs`（可拆 `src-tauri/src/agent/genai_model.rs`）

**Interfaces:**
- Produces: `struct GenaiChatModel { ... }` impl `ChatModel`
- `resolve_backend` → 扩展为携带 `protocol` 的 `ReadyLlm`（或直接返回 `Box<dyn ChatModel>` 工厂）

- [ ] **Step 1: `cargo add genai --package toolbox`**（版本写入 Cargo.lock）
- [ ] **Step 2: 单测** Anthropic + key → resolve 非 Unavailable；local 无模型 → Unavailable
- [ ] **Step 3: 实现适配器**：`ModelMessage` ↔ genai messages；tools JSON → genai tools；响应 → `ModelTurn`
- [ ] **Step 4: Agent 入口改用适配器**；`cargo test -p toolbox`
- [ ] **Step 5: 更新 `test_llm_connection` 按 protocol**

---

### Task 7: Regression + OpenSpec validate

- [ ] **Step 1: 手动 checklist** — local 无模型引导；GGUF 进度/成败；Ollama pull；协议切换后 test connection
- [ ] **Step 2: `openspec validate --change llm-multi-provider-protocols`**
- [ ] **Step 3: `cargo check -p toolbox` && 前端类型检查**

---

## Spec coverage self-check

| Spec requirement | Task |
|------------------|------|
| Provider Presets Aligned With CC Switch | 2, 3 |
| Selectable LLM Protocol | 1, 3, 6 |
| Ollama Provider And Model Pull | 5 |
| Download Progress Bar And Outcome Feedback | 4 |
| Switch Between Local and Cloud (modified) | 3, 6 |
| Protocol-Aware LLM Backend | 6 |
| LLM Unavailable Guidance (modified) | 6, 7 |

## Placeholder scan

无 TBD/TODO 占位；crate 版本以 `cargo add` 当时稳定版为准。
