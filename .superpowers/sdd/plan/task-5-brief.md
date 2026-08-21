# Task 2.3 — Mock LLM 的 Agent 循环

来源：plan.md Task 2.3；spec Agent Tool Loop

## Files

- Create: `src-tauri/src/agent/loop.rs`
- Create: `src-tauri/src/agent/llm.rs`
- Modify: `src-tauri/src/agent/mod.rs`
- 若需要落库助手消息：可在 `conversation.rs` 增加 `append_assistant_message_on`（不要改用户消息规则）
- 勾选 tasks.md 2.3

## Interfaces（verbatim 名称）

```rust
pub enum AgentEvent { Token(String), ToolStart { id: String, args: serde_json::Value }, ToolEnd { id: String, result: String }, Error(String), Done }
pub trait ChatModel { fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String>; }
pub enum ModelTurn { Text(String), ToolCalls(Vec<ToolCall>) }
pub fn run_agent(
    model: &mut impl ChatModel,
    conv_id: &str,
    user_text: &str,
    cancel: &std::sync::atomic::AtomicBool,
    emit: impl FnMut(AgentEvent),
) -> Result<(), String>
```

循环：retrieve_skills(user_text, 3) → model.complete → ToolCalls 则 registry::dispatch → 把 tool 结果推进 msgs → 直到 Text 或 cancel 或 **超过 8 次** tool 迭代。

禁止网络。测试用内存 Scripted mock。

## Tests（必须先写，RED）

1. 仅 Text
2. ToolCalls(base64.encode) 然后 Text
3. 两次 ToolCalls 然后 Text
4. dispatch 失败时下一轮模型仍被调用且收到错误字符串
5. cancel=true 时停止；已写入的 user 消息保留

`conv_id` 测试用 `append_user_message_on` 先建会话再 `run_agent`。

最终助手回复写入 messages role=assistant。中断时也可写空 assistant 或仅 Done/Error 事件——须在测试 5 中断言 user 消息仍在。

## Verify / Commit

`cargo test --lib agent::loop`；Windows ENTRYPOINT 问题时 `--no-run` + `cargo check`。

只 add agent loop/llm/mod、必要时 conversation.rs、tasks.md。Never `git add .`

`git commit -m "feat: run agent loop against a mock chat model"`
