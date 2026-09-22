# Sidebar Run Indicators & Parallel Conversation Runs — Design

Date: 2026-09-22  
Status: approved (brainstorming)  
Scope: product behavior change — requires OpenSpec change before implementation  
Suggested OpenSpec name: `parallel-conversation-runs`

## Goal

1. Multiple conversations can run Agent turns **in parallel**.
2. Left sidebar shows a **per-row leading icon**: idle = message icon; running = spinner/pulse.
3. Leaving a running conversation does **not** cancel it; returning **reconnects** to in-memory stream progress.
4. Cancel is available **only** for the currently open conversation (composer stop button).

## Decisions (confirmed)

| Topic | Choice |
|-------|--------|
| Overall approach | **B** — backend in-process run registry (state machine hub) |
| Sidebar icon states | **Two states only**: idle (message) / running (spinner or pulse) |
| Cancel UX | Stop button only when that conversation is open |
| Switch away | Background continues; switch back shows buffered progress + live stream if still running |
| Same conversation re-send while running | Reject with explicit error code `turn_in_progress` |
| Persistence of registry | Process memory only; restart ⇒ in-flight treated as interrupted (same as today) |

## Out of scope (this change)

- Sidebar cancel control
- Error / unread / “new reply” badges on sidebar rows
- Persisted SQLite run table or run history UI
- Global max-concurrency limit (no cap beyond one turn per conversation)
- Changing right-click menu (rename / trajectory / delete) beyond delete-while-running behavior below

## Architecture

### Run registry (Rust, in-process)

Central registry keyed by `conversation_id`:

| Field | Role |
|-------|------|
| `status` | `idle` \| `running` (UI consumes only these two) |
| `cancel` | Per-conversation cancel flag (`AtomicBool` or equivalent) |
| `started_at` | Optional; diagnostics / future use |

Rules:

- At most **one** in-flight turn per conversation.
- Different conversations do **not** block each other.
- `send_chat_turn` registers `running` before work starts; clears to `idle` on Done / Interrupted / fatal Error (and emits status event).
- Replacing today’s single global `AgentCancel` with per-conversation flags in the registry.

Internal fields may exist for later (error, last message) but **must not** drive sidebar UI in this change.

### Events

Keep existing `agent-event` payloads (already include `conversationId`). Stream tokens / reasoning / tools for **any** conversation id, whether or not it is the active UI thread.

Add `agent-run-status` (name fixed in OpenSpec tasks):

```json
{ "conversationId": "<id>", "status": "running" | "idle" }
```

Emit when a turn starts and when it becomes idle (Done, Interrupted, or fatal Error path).

### Frontend store

Pinia (or equivalent shared store) owns:

- `runStatusById: Record<string, 'idle' | 'running'>` — driven by `agent-run-status` (and hardened against missed events via turn lifecycle).
- `liveById: Record<string, LiveBuffer>` — per-conversation in-memory stream buffer (tokens, reasoning, tool steps, etc.).

Behaviors:

- **Open conversation**: render DB messages; if `liveById[id]` exists, overlay it; keep listening.
- **Switch away**: change `activeId` only; do not clear buffer; do not cancel.
- **Turn end**: persist as today; clear that id’s live buffer; status → idle.
- **Composer busy / stop**: derived from `runStatusById[activeId] === 'running'`.

### Sidebar UI

Row layout: `[leading icon][title]`.

- Idle: message icon (`fa-message`; fallback `fa-comment` if icon set lacks it).
- Running: `fa-spinner fa-spin` (or equivalent); color `var(--warning)` to match existing streaming cue.
- Whole row still opens the conversation; icon is not a separate control.
- Context menu, active styling, inline rename unchanged.

### Cancel API

- `cancel_chat_turn(conversationId: string)` — cancels **only** that conversation’s registry flag.
- Remove parameterless global cancel.
- UI calls it only for `activeId` when that id is running.

### Delete while running

If the user deletes a conversation that is `running`: **cancel that turn first**, then delete (so the worker does not emit into a gone conversation / orphan registry entry). Confirm dialog copy may stay as today.

## Error handling

| Case | Behavior |
|------|----------|
| Second send on same running conversation | Command returns error string/code `turn_in_progress`; UI shows a short error; no second registry entry |
| Cancel non-running / unknown id | No-op `Ok(())` |
| Background turn errors | Emit `agent-event` error + `agent-run-status` idle for that id; if conversation not open, user sees result when they open it (no sidebar error badge this change) |
| App restart mid-turn | Registry empty; no false “running” icons until a new turn starts |

## Testing / verification

1. A generating → switch to B → send on B: both sidebar rows show running; neither cancels the other.
2. Stop on B while viewing B: only B stops; A continues.
3. Return to A while A still running: see buffered text + continued streaming.
4. A completes: sidebar icon returns to message; live buffer for A cleared.
5. Send again on A while A running: rejected with clear feedback.
6. Delete running conversation: turn cancelled, row removed, no stuck running icon for that id.
7. Right-click rename / open trajectory / delete still work on idle and running rows (delete follows rule above).

## Files likely touched

**Rust**

- `src-tauri/src/commands/agent.rs` — registry, per-id cancel, status emit, in-progress guard
- Possibly small helper module e.g. `src-tauri/src/agent/run_registry.rs`
- `src-tauri/src/lib.rs` — state wiring if registry is managed state

**Frontend**

- `src/stores/conversations.ts` (or new `agentRuns` store) — `runStatusById` + `liveById`
- `src/views/HomePage.vue` — route events by id; busy from active run status
- `src/layout/SideBar.vue` — leading icons

**Specs**

- OpenSpec change under `openspec/changes/parallel-conversation-runs/` (propose → apply → archive)
- Living delta primarily on `agent-chat` (cancel signature, parallel runs, sidebar indicators)

## Implementation gate

This document is the Superpowers design record. **Do not implement until:**

1. User approves this file, and
2. An OpenSpec change is proposed (`/opsx-propose` or equivalent) aligned with this design, then
3. An implementation plan is written and executed under that change’s tasks.
