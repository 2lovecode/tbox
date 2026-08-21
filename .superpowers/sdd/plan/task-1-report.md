# Task 1.1 Report — 会话 SQLite 与纯函数 API

**Status:** DONE_WITH_CONCERNS  
**Branch:** `add-chat-home-agent`  
**Commit:** `fbf1e30` — feat: persist chat conversations on first user message

## Summary

实现了 `src-tauri/src/commands/conversation.rs`，提供会话与消息的 SQLite 持久化纯函数 API。首条用户消息才创建会话（标题 content 截断 ≤40 字符）；无空会话落库；删除会话级联移除消息。启动路径 `init_db_if_needed` 调用 `ensure_conversation_schema` 做幂等迁移。

## Files Changed

| File | Action |
|------|--------|
| `src-tauri/src/commands/conversation.rs` | Created |
| `src-tauri/src/commands/mod.rs` | Added `pub mod conversation;` |
| `src-tauri/src/commands/tool.rs` | Call `ensure_conversation_schema` in `init_db_if_needed` |
| `openspec/changes/add-chat-home-agent/tasks.md` | Checked 1.1 |

## Implementation Notes

- **Structs:** `Conversation`, `ChatMessage` per brief signatures.
- **Public API:** `append_user_message`, `list_conversations`, `get_messages`, `delete_conversation` — delegate to `open_connection()` + `*_on` helpers.
- **Test helpers:** `append_user_message_on`, `list_conversations_on`, `get_messages_on`, `delete_conversation_on`, `ensure_conversation_schema` — accept injected `&Connection`.
- **Schema:** `conversations` + `messages` with `ON DELETE CASCADE`; `PRAGMA foreign_keys = ON` on each connection use.
- **IDs:** `uuid::Uuid::new_v4()` strings.
- **No** `create_empty_conversation`.
- **Title:** `content.chars().take(40).collect()`.

## Self-Review

- [x] Spec: empty chat not persisted until first user message (`list_empty_without_any_message`).
- [x] Spec: first message creates conversation + title from content.
- [x] Spec: delete removes conversation and messages (CASCADE + test).
- [x] Tests use isolated temp DB under `std::env::temp_dir()`, never `~/.toolbox/tools.db`.
- [x] `mod.rs` only added `conversation`; no unrelated modules uncommented.
- [x] Commit scoped to brief files only (amended after accidental role-system hunks in `tool.rs`).

## TDD Evidence

### RED — tests before implementation

**Command:**
```
cd src-tauri && cargo test --lib commands::conversation
```

**Output (excerpt):**
```
error[E0425]: cannot find function `append_user_message_on` in this scope
error[E0425]: cannot find function `list_conversations_on` in this scope
error[E0425]: cannot find function `delete_conversation_on` in this scope
error[E0425]: cannot find function `get_messages_on` in this scope
error: could not compile `tbox` (lib test) due to 7 previous errors
```

Exit code: **101** (compile failure — expected RED).

### GREEN — after implementation

**Command:**
```
cd src-tauri && cargo test --lib commands::conversation --no-run
```

**Output (excerpt):**
```
Finished `test` profile [unoptimized + debuginfo] target(s) in 1.78s
Executable unittests src\lib.rs (target\debug\deps\tbox_lib-99fb53634424f68a.exe)
```

Exit code: **0** (tests compile).

**Runtime test execution:**
```
cd src-tauri && cargo test --lib commands::conversation
```

**Output (excerpt):**
```
Running unittests src\lib.rs (target\debug\deps\tbox_lib-99fb53634424f68a.exe)
error: test failed
Caused by:
  process didn't exit successfully: ... (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
```

Same failure affects **all** `cargo test --lib` targets (including existing `it_works` in `lib.rs`) — Windows/Tauri cdylib DLL entrypoint issue in this environment, not conversation-specific logic. Tests are structurally correct and compile.

**cargo check:**
```
cd src-tauri && cargo check
```
Exit code: **0**.

## Concerns

1. **Test runtime blocked:** `STATUS_ENTRYPOINT_NOT_FOUND` prevents executing lib tests on this Windows GNU/Tauri setup. Recommend verifying on CI or after WebView2/runtime PATH fix.
2. **Initial commit scope:** First commit accidentally staged role-removal hunks in `tool.rs`; amended to only `+2` lines before report.

## Next Task

1.2 — Register Tauri invoke commands wrapping the public API.

---

## Fix Report (Task 1.1 Review)

**Date:** 2026-08-21  
**Commit:** `ca02edb` — fix: transactional append for existing conversations

### Finding 1 — Transactional append for existing conversation

**Change:** Wrapped the `Some(conv_id)` branch of `append_user_message_on` in `unchecked_transaction()` — UPDATE `conversations.updated_at` and INSERT message now commit atomically (matches `None` branch).

**File:** `src-tauri/src/commands/conversation.rs`

### Finding 2 — Windows lib test execution (`STATUS_ENTRYPOINT_NOT_FOUND`)

**Workarounds attempted (minimal, no large refactor):**

| Attempt | Command / change | Result |
|---------|------------------|--------|
| `crate-type = ["rlib"]` only (drop cdylib) | `cargo test --lib commands::conversation` | Still `0xc0000139` |
| Integration test binary | `cargo test --test conversation` | Same `0xc0000139` before any test runs |
| Copy `WebView2Loader.dll` beside test exe | Manual run of `conversation-*.exe` | Same `0xc0000139` |

**Added:** `src-tauri/tests/conversation.rs` — mirrors unit tests + `append_existing_conversation_is_transactional`; uses temp SQLite file and `append_user_message_on`. Requires `pub mod commands` in `lib.rs` and `rusqlite` dev-dependency.

**Exact blocker:** On `x86_64-pc-windows-gnu`, any test binary that links `tbox_lib` (lib unit tests **and** integration tests) fails at **process startup** with Windows exit code `0xc0000139` (`STATUS_ENTRYPOINT_NOT_FOUND`) — before Rust test harness or `main` runs. Root cause is the Tauri 2 native dependency chain (`webview2-com-sys` / WebView2 loader DLLs), not conversation logic or cdylib name collision (renaming `tbox_lib.dll` did not help). Fixing this without a large refactor (e.g. making `tauri` optional behind a feature and gating all `#[tauri::command]` modules) is not feasible in this task scope.

### Covering tests — commands & output

**1. Lib unit tests (compile OK, runtime blocked):**
```
cd src-tauri && cargo test --lib commands::conversation
```
```
Finished `test` profile [unoptimized + debuginfo] target(s) in 9m 55s
Running unittests src\lib.rs (.../tbox_lib-99fb53634424f68a.exe)
error: test failed
Caused by:
  process didn't exit successfully: `...tbox_lib-99fb53634424f68a.exe commands::conversation` (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
```
Exit code: **3221225785** (0xc0000139). **Not a pass** — documented blocker.

**2. Integration tests (same blocker):**
```
cd src-tauri && cargo test --test conversation
```
```
Finished `test` profile [unoptimized + debuginfo] target(s) in 11m 25s
Running tests\conversation.rs (.../conversation-78aaf6c42d13e027.exe)
error: test failed
Caused by:
  process didn't exit successfully: `...conversation-78aaf6c42d13e027.exe` (exit code: 0xc0000139, STATUS_ENTRYPOINT_NOT_FOUND)
```
Exit code: **3221225785**. **Not a pass** — same environment blocker.

**3. Compile verification:**
```
cd src-tauri && cargo test --lib commands::conversation --no-run
cd src-tauri && cargo test --test conversation --no-run
```
Both exit **0** (tests compile and link).

### Files changed in fix

- `src-tauri/src/commands/conversation.rs` — transactional `Some` branch
- `src-tauri/tests/conversation.rs` — integration test suite (new)
- `src-tauri/src/lib.rs` — `pub mod commands` (integration test visibility)
- `src-tauri/Cargo.toml` — `[dev-dependencies] rusqlite` for integration test
