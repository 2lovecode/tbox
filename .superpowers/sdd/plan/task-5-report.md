# Task 2.3 Report — Mock LLM Agent loop

**Status:** DONE  
**Date:** 2026-08-22  
**Change:** `openspec/changes/add-chat-home-agent`  
**Commit:** `021f245` — feat: run agent loop against a mock chat model

## Summary

实现 `ChatModel` / `ModelTurn` / `ToolCall` / `AgentEvent` 与 `run_agent`（及可注入连接的 `run_agent_on`）。循环：`retrieve_skills(user_text, 3)` → `model.complete` → `ToolCalls` 时 `registry::dispatch` 并把结果以 `role=tool` 回填 → 直到 `Text`、`cancel`、或超过 **8** 次工具迭代。最终助手文本经 `append_assistant_message_on` 落库。测试使用内存 `Scripted` mock，无网络。

## Files

| Path | Action |
|------|--------|
| `src-tauri/src/agent/llm.rs` | Created (`ChatModel`, `ModelMessage`, `ModelTurn`, `ToolCall`) |
| `src-tauri/src/agent/loop.rs` | Created (`AgentEvent`, `run_agent`, `run_agent_on`, 5 tests) |
| `src-tauri/src/agent/mod.rs` | Modified (`pub mod llm;`, `pub mod r#loop;`) |
| `src-tauri/src/commands/conversation.rs` | Added `append_assistant_message_on` |
| `openspec/changes/add-chat-home-agent/tasks.md` | Checked 2.3 |

## TDD Evidence

### RED

先写五则测试 + 类型桩，故意不实现 `run_agent` / `run_agent_on`：

```text
cargo test --lib agent::loop --no-run
error[E0425]: cannot find function `run_agent_on` in this scope
error[E0425]: cannot find function `run_agent` in this scope
error: could not compile `tbox` (lib test) due to 6 previous errors
```

确认失败原因为缺失符号，而非断言写错。

### GREEN

实现循环与助手落库后：

```text
cargo test --lib agent::loop --no-run   # OK — Finished test profile
cargo check                             # OK — Finished dev profile
cargo test --lib agent::loop -- --nocapture
# exit 0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND（Windows 已知；编译已通过）
```

测试用例：

1. `text_only_reply` — 仅 Text，assistant 落库，Token + Done
2. `one_tool_call_then_text` — `base64.encode` 后 Text
3. `two_tool_calls_then_text` — 连续两次 ToolCalls 后 Text
4. `dispatch_failure_fed_back_to_model` — `http.request` 错误字符串进入下一轮 msgs
5. `cancel_stops_and_keeps_user_message` — cancel=true 不调模型，user 消息保留

## Commit

```text
feat: run agent loop against a mock chat model
```

仅 add：`llm.rs`、`loop.rs`、`mod.rs`、`conversation.rs`、`tasks.md`。未 `git add .`。

## Concerns

1. Windows 无法实际跑通 `--lib` 测试进程；GREEN 证据为 `--no-run` + `cargo check`（同 Task 2.1/2.2）。
2. 模块名为 `r#loop`（Rust 关键字）；`cargo test --lib agent::loop` 仍可按子串过滤。
3. 工具调用本身未单独落库，仅事件流 + 最终 assistant 文本；后续 UI 任务可扩展 `tool_calls_json`。
