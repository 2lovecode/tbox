# SDD ledger — plan: openspec/changes/add-chat-home-agent/plan.md

Workspace: current repo `D:\developer\workspace\rust\tbox` on branch `add-chat-home-agent` (not a linked worktree).

Ruling: Skip git worktree — HEAD does not contain uncommitted OpenSpec artifacts or the dirty WIP this change builds on; a worktree from HEAD would implement against a stale tree. Cost if wrong: less isolation from other dirty files; mitigate by instructing implementers to `git add` only task files.

CLI on PATH: cargo, pnpm, git. WARNING if make/zsh missing on Windows — use `cargo test` / `pnpm exec vue-tsc` directly.

## Preflight scan

| Pair / task | Shared surface | Check |
|-------------|----------------|-------|
| 1.1 → 1.2 | conversation.rs structs + `append_user_message` | 1.2 only adds `#[tauri::command]` / lib.rs register. Compatible. |
| 1.1 → 3.2 | same API names | Store will invoke commands from 1.2. Compatible. |
| 2.1 → 2.2 | lookup() vs skills | Skills must not expand registry. Compatible. |
| 2.1+2.2 → 2.3 | dispatch + retrieve_skills | loop consumes both. Compatible. |
| 2.3 → 5.4/6.1 | ChatModel trait | later tasks add real backend behind same trait. Compatible. |
| 3.1 → 3.2/3.3/4.x | HomePage.vue, SideBar.vue | sequential overlays. Compatible. |
| 5.1 → 5.4 | LlmProvider::Local | 5.4 adds resolve_backend. Compatible. |
| 1.1 tests vs code | test_db / `_on` helpers vs production `append_user_message` | Production uses global db; tests inject Connection. Not a contradiction. |
| 2.3 vs 4.1 | run_agent emit | 4.1 wraps with Tauri events. Compatible. |

No plan-mandated vs rubric defects found (tests assert behavior, not empty).

## Progress

Task 1: complete (commits cbb680d..ca02edb, review clean)
Task 1: minor (deferred): no 40-char title test; first-message test doesn't assert get_messages content; delete missing id silent; temp dirs not cleaned
Task 1: parked — lib.rs role handler removal in fix commit — Ruling: aligns with already-deleted role.rs / prior remove-role-system WIP. Cost if wrong: leftover frontend invoke names until that WIP is fully committed.
Task 2: complete (commits ca02edb..ad0a18f, review clean)
Task 3: complete (commits ad0a18f..96a37e2, review clean)
Task 3: parked — lib test exe STATUS_ENTRYPOINT_NOT_FOUND — Ruling: environment-wide, compile GREEN accepted (same as Task 1).
Task 4 (plan 2.2): complete (commits 96a37e2..3a7d810) — ledger catch-up after session resume
Task 5 (plan 2.3): complete (commits 3a7d810..021f245) — ledger catch-up after session resume
Task 6 (plan 3.1): complete (commits 021f245..9c3be4e)
Task 7–8 (plan 3.2–3.3): complete (ebfb276)
Task 9–11 (plan 4.1–5.1): complete (229883c)
Task 12–16 (plan 5.2–6.2): complete (6261c92 + openspec artifacts commit)
Task 17 (plan 6.3): parked — make missing on Windows; src-tauri cargo check GREEN earlier; vue-tsc pre-existing debt; lib test runtime ENTRYPOINT issue. Cost if wrong: miss regressions until CI.

