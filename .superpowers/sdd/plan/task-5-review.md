# Task 2.3 Review — Mock LLM Agent 循环

**Reviewer:** Code review gate (task-scoped)  
**Commit:** `021f245` — `feat: run agent loop against a mock chat model`  
**Sources:** `task-5-brief.md`, `task-5-report.md`, `task-5-review-pkg.txt`（静态 diff / 源码核对；本 gate 重跑 `cargo test --lib agent::loop --no-run`、`cargo check`、一次 lib test 执行）

---

## 结论摘要

实现与 Task 2.3 brief / plan 及 `Agent Tool Loop` 规格对齐：公开接口 `ChatModel` / `ModelTurn` / `ToolCall` / `AgentEvent` / `run_agent`（及可注入连接的 `run_agent_on`）名称与签名齐全；循环为 `retrieve_skills(user_text, 3)` → `model.complete` → `registry::dispatch` 回填 `role=tool` → 直至 `Text` / cancel / **超过 8 次**工具轮次；五则 Scripted 内存 mock 覆盖仅 Text / 一次 tool / 两次 tool / dispatch 失败回填 / cancel 保留 user；无网络客户端；助手终态经 `append_assistant_message_on` 落库。Windows 上 lib test 进程仍 `STATUS_ENTRYPOINT_NOT_FOUND`，与 Task 2.1/2.2 一致，编译 GREEN 可接受。

| 维度 | 裁决 |
|------|------|
| **Spec** | ✅ |
| **Quality** | **Approved** |

---

## Spec 符合性（对照 brief + 用户门禁 + Agent Tool Loop）

| 要求 | 结论 | 证据（diff / 源码） |
|------|------|---------------------|
| `AgentEvent` 五变体 verbatim | ✅ | `Token` / `ToolStart { id, args }` / `ToolEnd { id, result }` / `Error` / `Done` |
| `ChatModel::complete(&mut self, msgs) -> Result<ModelTurn, String>` | ✅ | `llm.rs` trait |
| `ModelTurn::{ Text, ToolCalls }` + `ToolCall { id, name, arguments }` | ✅ | `llm.rs` |
| `run_agent(model, conv_id, user_text, cancel, emit)` | ✅ | `loop.rs`；测试 `_signature_check` 绑定该签名；生产路径 `open_connection` → `run_agent_on` |
| 循环：`retrieve_skills` → complete → dispatch → 回填 msgs | ✅ | `build_system_prompt` 调 `retrieve_skills(..., 3)`；`ToolCalls` 分支 `registry::dispatch`；`ModelMessage::tool(result)` push |
| 直到 Text / cancel / **超过 8 次** tool 迭代 | ✅ | `MAX_TOOL_ITERATIONS = 8`；`tool_iterations >= 8` 时 Error+Done+Err；允许恰好 8 轮 tool |
| 禁止网络；Scripted 内存 mock | ✅ | `agent/` 无 HTTP 客户端；`Scripted` 仅 `VecDeque`；失败测用未注册 `http.request`（registry 拒绝，不发网） |
| 测试 1：仅 Text | ✅ | `text_only_reply`：assistant 落库 + Token + Done |
| 测试 2：一次 ToolCalls(base64.encode) 再 Text | ✅ | `one_tool_call_then_text`：ToolStart/End + `received.len()==2` |
| 测试 3：两次 ToolCalls 再 Text | ✅ | `two_tool_calls_then_text`：`base64.encode` → `hash.digest` → Text；`received.len()==3` |
| 测试 4：dispatch 失败仍调下一轮且 msgs 含错误串 | ✅ | `dispatch_failure_fed_back_to_model`：第二轮含 `role=tool` 且 content 含「未注册」/「unknown」 |
| 测试 5：cancel=true 停止；user 消息保留 | ✅ | `cancel_stops_and_keeps_user_message`：`received` 空；user 仍在；Done（或 Error） |
| 最终助手 `role=assistant` 落库 | ✅ | `append_assistant_message_on`；Text 路径写入正文 |
| 中断可空 assistant 或仅事件 | ✅ | 入口 cancel 仅 Done；环内 cancel 写空 assistant + Done；测 5 只强制 user 保留 |
| `append_user_message_on` 先建会话再跑 | ✅ | 五测均如此 |
| tasks.md 勾选 2.3 | ✅ | `- [x] 2.3 ...` |
| commit 范围与 message | ✅ | 5 文件：`llm.rs` / `loop.rs` / `mod.rs` / `conversation.rs` / `tasks.md`；message 与 brief 一致；未 `git add .` |

### 与 implementer report 的核对

| 声称 | 验证 |
|------|------|
| ChatModel / run_agent / 8 次上限 / Scripted 五测 | ✅ |
| RED 缺符号 → GREEN 实现 | ✅（依 report；本 gate 见完整实现） |
| Windows `--no-run` + `cargo check` | ✅ 本 gate：二者 exit 0 |
| Windows 无法跑通 lib test 进程 | ✅ 本 gate：`STATUS_ENTRYPOINT_NOT_FOUND` (0xc0000139) |
| 仅上述文件入 commit | ✅ `git show --stat 021f245` |

---

## 质量评估

### 做得好的地方

- **可测性**：`run_agent_on(conn, …)` 与应用 DB 解耦，五测全部走注入连接 + Scripted，符合「CI 不启 sidecar、不下载 GGUF」。
- **事件与落库分工清晰**：流式用 `AgentEvent`；终态助手文本落库；工具失败以 `ToolEnd.result` 字符串回填模型，不崩溃。
- **取消语义务实**：入口 `cancel=true` 不调模型、不丢已有 user；与 brief「也可仅 Done」一致。
- **Skill 注入路径正确**：system prompt 经 `retrieve_skills(user_text, 3)`，与 2.2 检索能力衔接且不扩注册表。
- **范围克制**：未改 registry/skills 行为；`append_assistant_message_on` 不碰用户消息规则。

### 发现项

#### Critical

*无*

#### Important

1. **Lib 测试在本机 Windows 未实际执行通过**  
   与 Task 2.1/2.2 相同：`cargo test --lib agent::loop` → `STATUS_ENTRYPOINT_NOT_FOUND`。`--no-run` 与 `cargo check` exit 0，静态审查与五测逻辑对齐；**运行时 GREEN 证据仍缺**。建议在可跑 lib test 的环境补跑  
   `cargo test --lib agent::loop -- --nocapture`。

#### Minor

1. **无「第 9 次 ToolCalls 触发上限」显式测试**  
   `MAX_TOOL_ITERATIONS` 与分支已实现，但五测均未压到 8。brief 门禁要求行为存在即可；补一则 Scripted 连发 9 轮 ToolCalls 可锁住 off-by-one。

2. **`ToolStart`/`ToolEnd` 的 `id` 使用 `call.name`，忽略 `ToolCall.id`**  
   与当前测试断言（`id == "base64.encode"`）一致，且规格强调展示「名称」；同工具多次调用时实例级关联会弱。后续 UI 流式任务可用 `call.id` 作事件相关 id、另带 name。

3. **模型 `complete` Err 时只发 `Error`、不发 `Done`**  
   超限路径为 Error+Done；模型失败路径无 Done，前端若只等 Done 可能挂起。本 task 无 UI，非阻塞；接入 4.x 流式前建议统一终态事件。

4. **`mod.rs` 缺文件末换行**  
   `pub mod r#loop;` 后无 newline；风格小瑕疵。

5. **环内 cancel 写空 assistant 时忽略 `append_assistant_message_on` 错误**（`let _ =`）  
   可接受为尽力落库；失败时仍 Done，调用方难区分。

---

## Gate 裁决

| 检查项 | 结果 |
|--------|------|
| Spec 全部 MUST（含用户门禁：run_agent / ChatModel / 五测 / max 8 / retrieve→dispatch / 无网络） | ✅ |
| Critical | 0 |
| Important | 1（环境级测试未跑通，非逻辑缺口） |
| **Quality** | **Approved** |

**Task 2.3 通过。** 可继续后续对话壳 / 流式 UI 任务；建议在能跑 `cargo test --lib` 的环境补跑 `agent::loop` 五测，并在接 UI 前统一 Error 路径的 `Done` 语义。
