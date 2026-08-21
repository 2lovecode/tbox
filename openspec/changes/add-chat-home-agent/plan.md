# 首页对话 Agent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. 本 schema **禁止** `executing-plans` fallback。
>
> 计划写在 `openspec/changes/add-chat-home-agent/plan.md`，不要写到 `docs/superpowers/plans/`。

**Goal:** 把 `/` 变成可持久化的对话首页，Rust Agent 用预置 Skill + 函数调用调度第一期纯计算工具，并支持设置页下载 GGUF、用 llama.cpp sidecar 作为默认本地 LLM（可切云端）。

**Architecture:** 前端只负责壳（侧栏历史、对话流、工具箱路由）和订阅流式事件。会话、Skill 检索、工具注册表、Agent 循环、LLM 路由（`local` sidecar 或现有云端）全在 Rust。本地权重不进安装包。CI 用 mock LLM，不下载 GGUF、不起 sidecar。

**Tech Stack:** Tauri 2, Vue 3, Pinia, SQLite (`~/.toolbox/tools.db`), 现有 `commands/llm.rs`, llama.cpp server sidecar, OpenAI-compatible chat+tools.

**Spec:** `openspec/changes/add-chat-home-agent/specs/agent-chat/spec.md`、`local-llm-runtime/spec.md`、以及 `product` / `tool-registry` / `local-ai-search` delta。设计见同目录 `design.md`。

## Global Constraints

- 产物与注释用中文；OpenSpec 标题与 SHALL/MUST 保持英文。
- API Key 不得出现在前端；只暴露 `hasApiKey`。
- Agent 工具白名单：id 9, 10, 11, 14（仅解析）, 16, 19, 20, 21, 30, 31, 32, 33；`side_effect` 全为 `none`。
- 某工具若目前只在 Vue 里算（例如 Base64 页），为本期 **新增薄 Rust command**，不改该工具页 UI。
- 单轮 Agent tool 循环上限：8。
- 验证命令：`cargo test`（在 `src-tauri`）、`cargo check`、`pnpm exec vue-tsc --noEmit`、`make check` / `make test`。
- 每完成 `tasks.md` 一项，把对应 `- [ ]` 改为 `- [x]`。

---

## Task 1.1: 会话 SQLite 与纯函数 API

**Files:**
- Create: `src-tauri/src/commands/conversation.rs`
- Modify: `src-tauri/src/commands/mod.rs`（`pub mod conversation`）
- Modify: `src-tauri/src/commands/tool.rs` 的 `init_db_if_needed`（或 `conversation` 内迁移，由 `lib.rs` 启动时调用）
- Test: `src-tauri/src/commands/conversation.rs` 内 `#[cfg(test)]`，用临时目录覆盖 DB 路径

**Interfaces:**
- Produces:
  - `pub struct Conversation { id: String, title: String, updated_at: i64 }`
  - `pub struct ChatMessage { id: String, conversation_id: String, role: String, content: String, tool_calls_json: Option<String>, created_at: i64 }`
  - `pub fn append_user_message(conversation_id: Option<String>, content: &str) -> Result<(Conversation, ChatMessage), String>` — `None` 时创建会话，标题为 content 截断（≤40 字）
  - `pub fn list_conversations() -> Result<Vec<Conversation>, String>`
  - `pub fn get_messages(conversation_id: &str) -> Result<Vec<ChatMessage>, String>`
  - `pub fn delete_conversation(conversation_id: &str) -> Result<(), String>`
  - **没有** `create_empty_conversation` 落库函数

- [ ] **Step 1: Write the failing test**

在 `conversation.rs` 测试里用 `tempfile` 或手动临时 `tools.db`。若项目未加 `tempfile`，测试内用 `std::env::temp_dir()` + 随机子目录，并提供 `with_test_db` 钩子。

```rust
#[test]
fn first_user_message_creates_conversation() {
    let db = test_db();
    let (conv, msg) = append_user_message_on(&db, None, "请把 hello 做 Base64").unwrap();
    assert!(!conv.id.is_empty());
    assert!(conv.title.contains("Base64") || conv.title.contains("hello"));
    assert_eq!(msg.role, "user");
    assert_eq!(list_conversations_on(&db).unwrap().len(), 1);
}

#[test]
fn delete_removes_messages() {
    let db = test_db();
    let (conv, _) = append_user_message_on(&db, None, "hi").unwrap();
    delete_conversation_on(&db, &conv.id).unwrap();
    assert!(list_conversations_on(&db).unwrap().is_empty());
    assert!(get_messages_on(&db, &conv.id).unwrap().is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test --lib commands::conversation::tests::first_user_message_creates_conversation -- --nocapture`

Expected: FAIL（模块/函数不存在）

- [ ] **Step 3: Write minimal implementation**

表结构：

```sql
CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  tool_calls_json TEXT,
  created_at INTEGER NOT NULL,
  FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
```

在 `init_db_if_needed` 或独立 `ensure_conversation_schema` 中执行。`id` 用 UUID 字符串。

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib commands::conversation`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/conversation.rs src-tauri/src/commands/mod.rs src-tauri/src/commands/tool.rs
git commit -m "feat: persist chat conversations on first user message"
```

勾选 `tasks.md` 1.1。

---

## Task 1.2: 注册会话 Tauri 命令

**Files:**
- Modify: `src-tauri/src/commands/conversation.rs`（`#[tauri::command]`）
- Modify: `src-tauri/src/lib.rs` `generate_handler!`

**Interfaces:**
- Produces 命令：`list_conversations`、`get_conversation_messages`、`delete_conversation`、`append_user_message`（参数 `conversationId: Option<String>`, `content: String`）

- [ ] **Step 1:** 给上述函数加 `#[tauri::command]`，在 `lib.rs` handler 列表注册。
- [ ] **Step 2:** `cd src-tauri && cargo check`
- [ ] **Step 3: Commit** `feat: expose conversation commands to the frontend`

勾选 `tasks.md` 1.2。

---

## Task 2.1: Agent 工具注册表

**Files:**
- Create: `src-tauri/src/agent/mod.rs`
- Create: `src-tauri/src/agent/registry.rs`
- Modify: `src-tauri/src/lib.rs`（`mod agent`）

**Interfaces:**
- Produces:
  - `pub enum SideEffect { None }`（预留未来变体）
  - `pub struct ToolSpec { pub id: &'static str, pub name: &'static str, pub schema: serde_json::Value, pub side_effect: SideEffect }`
  - `pub fn lookup(tool_id: &str) -> Option<&'static ToolSpec>`
  - `pub fn dispatch(tool_id: &str, args: &serde_json::Value) -> Result<String, String>`
  - 注册 id 使用稳定字符串，与工具数字 id 对应，例如 `json.format`、`base64.encode`、`hash.digest`、`jwt.parse`、`timestamp.convert`、`encoding.convert`、`xml.format`、`yaml.format`、`uuid.generate`、`cron.explain`、`number.convert`、`charset.convert`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn unknown_tool_is_rejected() {
    let err = dispatch("http.request", &json!({})).unwrap_err();
    assert!(err.contains("未注册") || err.contains("unknown"));
}

#[test]
fn invalid_args_do_not_run() {
    let err = dispatch("base64.encode", &json!({})).unwrap_err();
    assert!(err.contains("schema") || err.contains("参数"));
}

#[test]
fn base64_roundtrip() {
    let encoded = dispatch("base64.encode", &json!({"input": "hi"})).unwrap();
    let decoded = dispatch("base64.decode", &json!({"input": encoded})).unwrap();
    assert_eq!(decoded, "hi");
}
```

- [ ] **Step 2:** `cargo test --lib agent::registry` Expected: FAIL
- [ ] **Step 3:** 实现 schema（`jsonschema` crate 或手写必填字段检查，优先手写必填以免新依赖过重）+ `dispatch` 调用现有 command。缺 Rust 实现的工具（若 Base64 仅在前端）在 `encoding.rs` 增加：

```rust
#[tauri::command]
pub fn base64_encode(input: String) -> String {
    general_purpose::STANDARD.encode(input.as_bytes())
}
```

JWT **只**走解析（header/payload），不要默认验签。不要把 `commands::network` / `db_tools` 放进 `lookup`。

- [ ] **Step 4:** `cargo test --lib agent::registry` Expected: PASS；`cargo check`
- [ ] **Step 5: Commit** `feat: add allowlisted agent tool registry`

勾选 `tasks.md` 2.1。

---

## Task 2.2: 预置 Skill 检索

**Files:**
- Create: `src-tauri/skills/*.md`（每工具一份，front matter 含 `tool_id` 与 `keywords`）
- Create: `src-tauri/src/agent/skills.rs`
- Include 技能文件：`include_str!` 或 `include_dir`；第一期打进二进制即可

**Interfaces:**
- Produces: `pub fn retrieve_skills(query: &str, limit: usize) -> Vec<SkillDoc>` where `SkillDoc { tool_id, body }`
- `limit` 默认 3

- [ ] **Step 1: Failing test**

```rust
#[test]
fn jwt_query_hits_jwt_skill() {
    let hits = retrieve_skills("帮我解析这段 JWT", 3);
    assert!(hits.iter().any(|s| s.tool_id.contains("jwt")));
    assert!(hits.len() <= 3);
}

#[test]
fn loading_skills_does_not_register_tools() {
    let before = lookup("http.request");
    let _ = retrieve_skills("http", 3);
    assert!(before.is_none());
    assert!(lookup("http.request").is_none());
}
```

- [ ] **Step 2:** 运行测试 Expected: FAIL
- [ ] **Step 3:** 关键词匹配（工具名、中文别名、tags）。JWT Skill 写明：只解析，不验签。
- [ ] **Step 4:** 测试 PASS
- [ ] **Step 5: Commit** `feat: retrieve preset tool skills for agent context`

勾选 `tasks.md` 2.2。

---

## Task 2.3: Mock LLM 的 Agent 循环

**Files:**
- Create: `src-tauri/src/agent/loop.rs`
- Create: `src-tauri/src/agent/llm.rs`（`trait ChatModel`）

**Interfaces:**
- Produces:
  - `pub enum AgentEvent { Token(String), ToolStart { id, args }, ToolEnd { id, result }, Error(String), Done }`
  - `pub trait ChatModel { fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String>; }`
  - `pub enum ModelTurn { Text(String), ToolCalls(Vec<ToolCall>) }`
  - `pub fn run_agent(model: &mut impl ChatModel, conv_id: &str, user_text: &str, cancel: &AtomicBool, emit: impl FnMut(AgentEvent)) -> Result<(), String>`
  - 循环：`retrieve_skills` → 调模型 → tool call 则 `dispatch` → 把 tool 结果推进 `msgs` → 直到 `Text` 或 cancel 或超过 8 次

- [ ] **Step 1: Failing tests**（全部用内存 mock，禁止网络）

```rust
struct Scripted(Vec<ModelTurn>);
// 1) 仅 Text
// 2) ToolCalls(base64.encode) 然后 Text
// 3) 两次 ToolCalls 然后 Text
// 4) dispatch 失败时下一轮模型仍被调用且收到错误字符串
// 5) cancel=true 时停止并留下已写入的 user 消息
```

- [ ] **Step 2:** `cargo test --lib agent::loop` Expected: FAIL
- [ ] **Step 3:** 实现 `run_agent`；助手最终回复 `append` 到 messages（role=assistant）；中断时 role=assistant 且 content 可为空、或加 `status=interrupted` 字段（若加字段，同步改 Task 1.1 表，允许 `status TEXT`）。
- [ ] **Step 4:** 测试 PASS
- [ ] **Step 5: Commit** `feat: run agent loop against a mock chat model`

勾选 `tasks.md` 2.3。

---

## Task 3.1: `/toolbox` 与对话空态壳

**Files:**
- Create: `src/views/ToolboxPage.vue`（从 `HomePage.vue` 搬网格与分类过滤）
- Modify: `src/views/HomePage.vue` 改为对话空态（输入框可先 disabled）
- Modify: `src/layout/SideBar.vue`：新建对话、历史占位、底部工具箱
- Modify: `src/router/main.ts`：`/toolbox` → ToolboxPage；`/` → HomePage
- Modify: 任何 `router.push({ path: '/' })` 且意图是「看工具」的调用改为 `/toolbox`（分类点击）

**Interfaces:**
- 侧栏 `goToolbox()` → `/toolbox`
- Spotlight 工具导航 **保持原工具路由**，不要改成 `/`

- [ ] **Step 1:** 搬迁组件，保证原网格 class/行为仍在 ToolboxPage。
- [ ] **Step 2:** `pnpm exec vue-tsc --noEmit`
- [ ] **Step 3:** 手动：`/` 无工具卡片；工具箱打开 Base64（`/base64-tool`）。
- [ ] **Step 4: Commit** `feat: move tool grid to /toolbox and show chat shell on home`

勾选 `tasks.md` 3.1。

---

## Task 3.2: 接会话列表

**Files:**
- Create: `src/stores/conversations.ts`
- Modify: `SideBar.vue`、`HomePage.vue`

**Interfaces:**
- Store: `items`, `activeId`, `draftId`（仅前端，未 `append_user_message` 前不进 `items`）
- `newChat()` 只设 `draftId` + 清空消息
- `sendFirst(content)` → `invoke('append_user_message', { conversationId: null, content })` 然后用返回 id 替换 draft

- [ ] **Step 1:** store + 侧栏绑定 list/delete
- [ ] **Step 2:** `vue-tsc`
- [ ] **Step 3:** 手动：空新建不出现在历史；发一条后出现标题；删除消失
- [ ] **Step 4: Commit** `feat: wire conversation list and first-message persist`

勾选 `tasks.md` 3.2。

---

## Task 3.3: 返回首页恢复会话

**Files:**
- Modify: `src/stores/conversations.ts`（`lastActiveId` 持久到 `sessionStorage` 即可，不必进 SQLite）
- Modify: `HomePage.vue` onMounted 加载 `get_conversation_messages`

- [ ] **Step 1:** 实现恢复
- [ ] **Step 2:** 手动：打开工具页再回 `/` 仍是刚才的对话
- [ ] **Step 3: Commit** `feat: restore last chat when returning home`

勾选 `tasks.md` 3.3。

---

## Task 4.1: 流式事件 UI

**Files:**
- Create: `src-tauri/src/commands/agent.rs`（`send_chat_turn`, `cancel_chat_turn`）
- Modify: `lib.rs` handler + 使用 `app.emit("agent-event", payload)`
- Modify: `HomePage.vue` 监听 `agent-event`

**Interfaces:**
- 事件 payload: `{ conversationId, type: "token"|"tool_start"|"tool_end"|"error"|"done"|"interrupted", ... }`
- 此步可继续接 **mock/scripted model**（设置环境变量或 debug 开关），以便无 LLM 时开发 UI。真实路由在 6.1。

- [ ] **Step 1:** Rust emit + 前端渲染工具卡片
- [ ] **Step 2:** `cargo check` && `vue-tsc`
- [ ] **Step 3:** 手动或 mock：一次 tool call 卡片出现；点取消后状态为中断
- [ ] **Step 4: Commit** `feat: stream agent tokens and tool calls to the chat ui`

勾选 `tasks.md` 4.1。

---

## Task 4.2: LLM 不可用引导

**Files:**
- Modify: `commands/agent.rs` 在无可用模型时返回明确错误码 `llm_unavailable`
- Modify: `HomePage.vue` 展示「去设置下载模型或配置云端」并打开 `SettingsModal` 的 llm tab

- [ ] **Step 1:** 失败路径测试（Rust：`run_agent` 前检查 provider 就绪）
- [ ] **Step 2:** UI 文案与按钮
- [ ] **Step 3:** 手动：无模型无云端发送 → 看到设置引导，无崩溃
- [ ] **Step 4: Commit** `feat: prompt users to configure llm when agent cannot run`

勾选 `tasks.md` 4.2。

---

## Task 5.1: `LlmProvider::Local` 默认值

**Files:**
- Modify: `src-tauri/src/commands/llm.rs`（enum 增加 `Local`，`Default` 改为 `Local`）
- Modify: `src/types/llm.ts`、`src/stores/llm.ts`、`SettingsModal.vue`
- Test: `get_llm_config` 在无文件时 provider=`local`；有文件时原样反序列化

**注意：** 现有 `Default` 是 `Openai`。改为 `Local` 只影响 **缺文件** 路径；`load` 已有 json 时保持字段。

```rust
#[test]
fn missing_config_defaults_to_local() {
    assert_eq!(LlmProvider::default(), LlmProvider::Local);
}
```

- [ ] **Step 1:** 失败测试（Default 仍是 Openai 则 assert 失败）
- [ ] **Step 2:** 改 enum + 前端选项「本地（需下载模型）」
- [ ] **Step 3:** `cargo test` llm 相关 + `vue-tsc`
- [ ] **Step 4: Commit** `feat: add local llm provider as default for new installs`

勾选 `tasks.md` 5.1。

---

## Task 5.2: 精选 GGUF 下载

**Files:**
- Create: `src-tauri/src/commands/model_catalog.rs`
- Modify: settings UI 新区块
- 清单常量：推荐 Qwen2.5-1.5B-Instruct Q4_K_M（实现时写入确定 URL + sha256）。备选一项同系列更小模型。

**Interfaces:**
- `list_local_models() -> Vec<{ id, label, recommended, installed, sizeBytes }>`
- `start_model_download(id)` / `cancel_model_download(id)` + 事件 `model-download-progress { id, received, total }`
- 完成后写 `~/.toolbox/models/<id>.gguf` + sidecar checksum 文件；不完整则删除临时 `.part`

- [ ] **Step 1:** 用 `mockito` 或本地 `std::net::TcpListener` 假 HTTP 测成功/取消/失败。**禁止 CI 访问 huggingface.co。**
- [ ] **Step 2:** 实现下载到 `.part` 再 rename；取消删除 `.part`
- [ ] **Step 3:** 测试 PASS；`vue-tsc`
- [ ] **Step 4: Commit** `feat: download curated local gguf models from settings`

勾选 `tasks.md` 5.2。

---

## Task 5.3: sidecar 生命周期

**Files:**
- Create: `src-tauri/src/agent/sidecar.rs`
- Modify: `tauri.conf.json`（或 v2 等价配置）声明 sidecar 名如 `llama-server`
- 二进制稍后由打包脚本提供；单测用 `trait ProcessCtl`

**Interfaces:**
- `pub struct SidecarConfig { host: "127.0.0.1", port: u16 }` port **默认 11435**（禁止 11434）
- `start(model_path) -> Result<SidecarHandle, String>`
- `stop()`
- `health() -> bool`
- 主进程 `RunEvent::Exit` 时 `stop`

- [ ] **Step 1:** 状态机单测：start 记录 bind=`127.0.0.1:11435`；模拟进程退出 → `health()==false` 且不 panic
- [ ] **Step 2:** 实现；Windows 注意 Job Object 或 `kill_on_drop` 以免孤儿进程
- [ ] **Step 3:** `cargo test --lib agent::sidecar`
- [ ] **Step 4: Commit** `feat: manage llama.cpp sidecar on loopback`

勾选 `tasks.md` 5.3。

---

## Task 5.4: 本地/云端路由守卫

**Files:**
- Modify: `src-tauri/src/agent/llm.rs` 增加 `ReadyLlm` 枚举
- `fn resolve_backend(cfg: &LlmConfig, local_model: Option<PathBuf>) -> Result<ReadyLlm, AgentError>`
  - `Local` + 无模型 → `Err(llm_unavailable)` **不得** fallback 到云端
  - `Openai|Deepseek|Anthropic|Custom` → HTTP 客户端（Key 只在 Rust）

- [ ] **Step 1:**

```rust
#[test]
fn local_without_model_does_not_hit_cloud() {
    let cfg = LlmConfig { provider: LlmProvider::Local, ..default_cloud_looking_urls() };
    let err = resolve_backend(&cfg, None).unwrap_err();
    assert!(matches!(err, AgentError::LlmUnavailable));
}
```

- [ ] **Step 2:** 实现 + 设置里切换后 `save_llm_config`
- [ ] **Step 3:** 测试 PASS
- [ ] **Step 4: Commit** `feat: refuse silent cloud fallback when local model missing`

勾选 `tasks.md` 5.4。

---

## Task 6.1: 真实 Chat Completions + tool 接入循环

**Files:**
- Modify: `src-tauri/src/agent/llm.rs` — OpenAI 兼容 `POST /v1/chat/completions` with `tools` array from registry schemas；Anthropic 用其 tools 协议或第一期对 Anthropic **文档化为仅当 Custom/OpenAI 兼容时保证 tool call**（若 Anthropic 映射工作量过大：设置里注明「Agent 工具调用推荐 OpenAI 兼容或本地」，Anthropic 仅纯文本——**优先做完 OpenAI 兼容 + local sidecar**，Anthropic tool 映射作为本任务内若超 1 小时则在 tasks 注明并保持纯文本，避免静默谎称支持）。
- Sidecar 启动后 `base_url=http://127.0.0.1:11435/v1`，model=启用的文件 stem。
- `send_chat_turn` 使用 `resolve_backend` + `run_agent`。

- [ ] **Step 1:** 用 mock HTTP 测：模型返回 tool_calls JSON → dispatch → 第二次请求带 tool role
- [ ] **Step 2:** 接入；迭代上限 8
- [ ] **Step 3:** `cargo check`；手动 Base64 对话（云端或 sidecar）
- [ ] **Step 4: Commit** `feat: route agent turns through local sidecar or cloud llm`

勾选 `tasks.md` 6.1。

---

## Task 6.2: living specs 表

**Files:**
- Modify: `AGENTS.md` Living specs 表增加：
  - 对话 Agent → `openspec/specs/agent-chat/`（归档后才有主 spec；实现期可写「将随 archive 进入」或先不新增目录）
  - 本地 LLM 运行时 → `openspec/specs/local-llm-runtime/`

在 **archive 之前** 主 specs 目录可能尚无这两份；本任务只改表格预告，或在注释写「apply 完成后 archive」。不要在未 archive 时手写一份与 delta 冲突的主 spec。

- [ ] **Step 1:** 编辑 `AGENTS.md`
- [ ] **Step 2:** 确认未把规格写进 `docs/superpowers/`
- [ ] **Step 3: Commit** `docs: list agent-chat and local-llm-runtime in living specs map`

勾选 `tasks.md` 6.2。

---

## Task 6.3: 全量验证

- [ ] **Step 1:** `make test`
- [ ] **Step 2:** `make check`
- [ ] **Step 3:** 手动冒烟清单
  - `/` 是对话；`/toolbox` 有分类卡片；点 Base64 进原页
  - Spotlight 能打开哈希工具
  - 无模型发送出现设置引导
  - （可选）云端或已下模型：对话调用 Base64
  - sidecar 若启动：只监听 127.0.0.1，不是 11434
- [ ] **Step 4:** 勾选 `tasks.md` 6.3。无新代码则不必空 commit。

---

## Spec coverage（self-review）

| Requirement | Task |
| Conversation Homepage | 3.1, 3.3 |
| Conversation History | 1.1, 1.2, 3.2 |
| Agent Tool Loop | 2.3, 4.1, 6.1 |
| Allowlisted tools | 2.1 |
| Preset Skills | 2.2 |
| LLM Unavailable | 4.2, 5.4 |
| Local provider / no bundled weights | 5.1 |
| Curated download | 5.2 |
| Sidecar lifecycle | 5.3 |
| Switch local/cloud | 5.4, 6.1 |
| product / tool-registry / Spotlight 不强制 LLM | 3.1, 6.3 |

## 本计划不做

- 用户安装 Skill、MCP、GPU、任意 GGUF、会话搜索、副作用工具确认框（无注册即无调用）。
