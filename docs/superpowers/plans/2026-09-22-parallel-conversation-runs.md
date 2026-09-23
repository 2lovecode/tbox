# Parallel Conversation Runs & Sidebar Indicators Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Allow multiple conversations to run Agent turns in parallel, with a backend run registry, per-conversation cancel, sidebar idle/running icons, and reconnect-to-stream when switching back.

**Architecture:** In-process Rust `RunRegistry` (per-`conversation_id` status + cancel flag) replaces global `AgentCancel`. Emit `agent-run-status` alongside existing `agent-event`. Frontend `agentRuns` store keeps `runStatusById` + `liveById` buffers so non-active conversations still accumulate stream events; SideBar reads status for leading icons; HomePage derives busy/stop from the active id.

**Tech Stack:** Tauri 2 + Rust (`Arc`/`Mutex`/`AtomicBool`), Vue 3 + Pinia, existing `agent-event` listen path, Font Awesome icons.

**Spec:** `docs/superpowers/specs/2026-09-22-sidebar-run-indicators-design.md`

## Global Constraints

- Behavior change → OpenSpec change `parallel-conversation-runs` MUST exist and be approved before code Tasks 2+ (AGENTS.md).
- Sidebar icons: idle = `fa-message` (fallback `fa-comment`); running = `fa-spinner fa-spin` with `var(--warning)`.
- Cancel only via composer stop for the open conversation; no sidebar stop.
- Same conversation while `running` → reject with error code/string `turn_in_progress`.
- Cancel unknown/idle id → `Ok(())` no-op.
- Delete running conversation → cancel that id first, then delete.
- No SQLite run table; registry is process memory only.
- Do not commit unless the user explicitly asks.
- Do not expand into unread/error badges or global concurrency caps.

---

## File map

| File | Responsibility |
|------|----------------|
| `openspec/changes/parallel-conversation-runs/*` | OpenSpec proposal / design / delta specs / tasks |
| `src-tauri/src/agent/run_registry.rs` | In-process registry: try_begin / cancel / finish / is_running |
| `src-tauri/src/agent/mod.rs` | `pub mod run_registry` |
| `src-tauri/src/commands/agent.rs` | Wire registry into send/cancel; emit `agent-run-status`; remove global `AgentCancel` |
| `src-tauri/src/lib.rs` | `.manage(RunRegistry::default())` instead of `AgentCancel` |
| `src/stores/agentRuns.ts` | `runStatusById`, `liveById`, event appliers |
| `src/views/HomePage.vue` | Route all `agent-event` into store; overlay live for active; cancel with id |
| `src/layout/SideBar.vue` | Leading icon per row from `runStatusById` |
| `src/stores/conversations.ts` | On delete: cancel-if-running then delete; clear run state for id |

---

### Task 1: OpenSpec change `parallel-conversation-runs`

**Files:**
- Create: `openspec/changes/parallel-conversation-runs/.openspec.yaml`
- Create: `openspec/changes/parallel-conversation-runs/proposal.md`
- Create: `openspec/changes/parallel-conversation-runs/design.md`
- Create: `openspec/changes/parallel-conversation-runs/tasks.md`
- Create: `openspec/changes/parallel-conversation-runs/specs/agent-chat/spec.md`

**Interfaces:**
- Consumes: design doc at `docs/superpowers/specs/2026-09-22-sidebar-run-indicators-design.md`
- Produces: apply-ready OpenSpec change whose delta matches that design (parallel runs, `cancel_chat_turn(conversationId)`, `agent-run-status`, sidebar two-state icons, reconnect buffer)

- [ ] **Step 1: Create change scaffold**

Run:

```bash
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox
openspec new change "parallel-conversation-runs"
openspec status --change "parallel-conversation-runs" --json
```

Expected: change directory exists; `applyRequires` lists artifacts to complete (typically proposal → specs → design → tasks).

- [ ] **Step 2: Fill artifacts via openspec-propose skill (or manually)**

Follow `.claude/skills/openspec-propose/SKILL.md` / `openspec instructions` for each pending artifact. Content MUST encode:

- **proposal Why/What:** multi-session parallel Agent turns; sidebar running indicator; per-conversation cancel; switch-away continues; switch-back reconnects live buffer.
- **delta `agent-chat` requirements** covering at least:
  - Parallel turns across conversations
  - `cancel_chat_turn` requires `conversationId`
  - `turn_in_progress` when re-sending same running conversation
  - `agent-run-status` event `running|idle`
  - Sidebar leading icon idle/running
  - Delete running conversation cancels first
- **design.md:** registry approach B (can summarize / point at Superpowers design; do not contradict it)
- **tasks.md:** mirror Tasks 2–7 of this plan at OpenSpec granularity

- [ ] **Step 3: Validate**

Run:

```bash
openspec validate --change "parallel-conversation-runs"
openspec status --change "parallel-conversation-runs"
```

Expected: validation passes; all apply-required artifacts `done`.

- [ ] **Step 4: Stop for user ack if needed**

If propose workflow says wait before apply: present change path and wait. Otherwise proceed to Task 2 only when user asks to implement / apply.

---

### Task 2: Rust `RunRegistry` (TDD)

**Files:**
- Create: `src-tauri/src/agent/run_registry.rs`
- Modify: `src-tauri/src/agent/mod.rs` — add `pub mod run_registry;`
- Test: unit tests inside `run_registry.rs` (`#[cfg(test)]`)

**Interfaces:**
- Consumes: `std::sync::{Arc, Mutex}`, `std::sync::atomic::{AtomicBool, Ordering}`, `std::collections::HashMap`
- Produces:
  - `pub struct RunRegistry { /* inner Mutex<HashMap<String, RunEntry>> */ }`
  - `pub struct RunEntry { pub cancel: Arc<AtomicBool>, pub started_at: u64 }` (started_at = unix secs optional)
  - `impl RunRegistry`:
    - `pub fn try_begin(&self, conversation_id: &str) -> Result<Arc<AtomicBool>, String>` — Err(`"turn_in_progress"`) if already running; else insert, return cancel flag reset to false
    - `pub fn cancel(&self, conversation_id: &str)` — if present, store true on flag; else no-op
    - `pub fn finish(&self, conversation_id: &str)` — remove entry (idle)
    - `pub fn is_running(&self, conversation_id: &str) -> bool`
  - `impl Default for RunRegistry`

- [ ] **Step 1: Write failing tests**

Create `src-tauri/src/agent/run_registry.rs` with tests first (stub `RunRegistry` empty if needed so file compiles, or write tests + `todo!` methods):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_begin_then_second_fails() {
        let reg = RunRegistry::default();
        let flag = reg.try_begin("c1").expect("first begin");
        assert!(!flag.load(Ordering::SeqCst));
        assert!(reg.is_running("c1"));
        let err = reg.try_begin("c1").unwrap_err();
        assert_eq!(err, "turn_in_progress");
    }

    #[test]
    fn parallel_conversations_ok() {
        let reg = RunRegistry::default();
        reg.try_begin("a").unwrap();
        reg.try_begin("b").unwrap();
        assert!(reg.is_running("a") && reg.is_running("b"));
    }

    #[test]
    fn cancel_sets_flag_finish_clears() {
        let reg = RunRegistry::default();
        let flag = reg.try_begin("c1").unwrap();
        reg.cancel("c1");
        assert!(flag.load(Ordering::SeqCst));
        reg.finish("c1");
        assert!(!reg.is_running("c1"));
        reg.cancel("missing"); // no panic
    }
}
```

- [ ] **Step 2: Run tests — expect FAIL**

Run:

```bash
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox/src-tauri
cargo test --lib agent::run_registry -- --nocapture
```

Expected: compile error or FAIL until implementation exists.

- [ ] **Step 3: Implement registry**

```rust
//! In-process agent run registry: one in-flight turn per conversation.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub const TURN_IN_PROGRESS: &str = "turn_in_progress";

pub struct RunEntry {
    pub cancel: Arc<AtomicBool>,
    pub started_at: u64,
}

pub struct RunRegistry {
    inner: Mutex<HashMap<String, RunEntry>>,
}

impl Default for RunRegistry {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl RunRegistry {
    pub fn try_begin(&self, conversation_id: &str) -> Result<Arc<AtomicBool>, String> {
        let mut map = self.inner.lock().map_err(|e| e.to_string())?;
        if map.contains_key(conversation_id) {
            return Err(TURN_IN_PROGRESS.to_string());
        }
        let cancel = Arc::new(AtomicBool::new(false));
        map.insert(
            conversation_id.to_string(),
            RunEntry {
                cancel: Arc::clone(&cancel),
                started_at: now_secs(),
            },
        );
        Ok(cancel)
    }

    pub fn cancel(&self, conversation_id: &str) {
        if let Ok(map) = self.inner.lock() {
            if let Some(entry) = map.get(conversation_id) {
                entry.cancel.store(true, Ordering::SeqCst);
            }
        }
    }

    pub fn finish(&self, conversation_id: &str) {
        if let Ok(mut map) = self.inner.lock() {
            map.remove(conversation_id);
        }
    }

    pub fn is_running(&self, conversation_id: &str) -> bool {
        self.inner
            .lock()
            .map(|m| m.contains_key(conversation_id))
            .unwrap_or(false)
    }
}
```

Add to `src-tauri/src/agent/mod.rs`:

```rust
pub mod run_registry;
```

- [ ] **Step 4: Run tests — expect PASS**

```bash
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox/src-tauri
cargo test --lib agent::run_registry -- --nocapture
```

Expected: all three tests PASS.

---

### Task 3: Wire `send_chat_turn` / `cancel_chat_turn` + `agent-run-status`

**Files:**
- Modify: `src-tauri/src/commands/agent.rs`
- Modify: `src-tauri/src/lib.rs` (manage `RunRegistry`, drop `AgentCancel`)

**Interfaces:**
- Consumes: `crate::agent::run_registry::{RunRegistry, TURN_IN_PROGRESS}`
- Produces:
  - Event name constant `pub const AGENT_RUN_STATUS: &str = "agent-run-status";`
  - Payload `{ conversationId: String, status: "running" | "idle" }` (serde camelCase)
  - `cancel_chat_turn(registry, conversationId: String) -> Result<(), String>`
  - `send_chat_turn` uses `registry.try_begin`; on any terminal path calls `finish` + emit idle

- [ ] **Step 1: Replace managed state in `lib.rs`**

Change:

```rust
.manage(commands::agent::AgentCancel::default())
```

to:

```rust
.manage(crate::agent::run_registry::RunRegistry::default())
```

- [ ] **Step 2: Rewrite cancel + status helpers in `agent.rs`**

Remove `struct AgentCancel`. Add:

```rust
use crate::agent::run_registry::{RunRegistry, TURN_IN_PROGRESS};

pub const AGENT_RUN_STATUS: &str = "agent-run-status";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunStatusPayload {
    pub conversation_id: String,
    pub status: String, // "running" | "idle"
}

fn emit_run_status(app: &AppHandle, conversation_id: &str, status: &str) {
    let _ = app.emit(
        AGENT_RUN_STATUS,
        AgentRunStatusPayload {
            conversation_id: conversation_id.to_string(),
            status: status.to_string(),
        },
    );
}
```

`cancel_chat_turn`:

```rust
#[tauri::command]
#[allow(non_snake_case)]
pub fn cancel_chat_turn(
    registry: State<'_, RunRegistry>,
    conversationId: String,
) -> Result<(), String> {
    registry.cancel(&conversationId);
    Ok(())
}
```

- [ ] **Step 3: Update `send_chat_turn` lifecycle**

At start (after `llm_is_ready()?`):

```rust
let cancel = registry.try_begin(&conversationId)?;
emit_run_status(&app, &conversationId, "running");
```

Inside `spawn_blocking`, wrap the existing `run_agent` / error paths so **every** exit does:

```rust
registry.finish(&conv_id);
emit_run_status(&app, &conv_id, "idle");
```

Pass `cancel.as_ref()` into `run_agent` (same as today’s flag). Clone `AppHandle` and use `State` owned `Arc` pattern: manage `RunRegistry` behind the same type; for spawn thread either:

- `let registry = app.state::<RunRegistry>()` is not available off main — prefer `RunRegistry` stored as `Arc<RunRegistry>` managed, or clone needed handles before spawn.

Practical pattern matching existing cancel clone:

```rust
// Manage Arc<RunRegistry> OR keep RunRegistry and clone Arc flags only.
// try_begin already returns Arc<AtomicBool> for the worker.
// For finish from worker: hold Arc<RunRegistry>.
```

Update `lib.rs` to:

```rust
.manage(std::sync::Arc::new(crate::agent::run_registry::RunRegistry::default()))
```

And command signatures use `State<'_, Arc<RunRegistry>>`. Inside `send_chat_turn`:

```rust
let registry = Arc::clone(&registry);
let cancel = registry.try_begin(&conversationId)?;
emit_run_status(&app, &conversationId, "running");
let app2 = app.clone();
let conv_id = conversationId.clone();
tauri::async_runtime::spawn_blocking(move || {
    let finish = || {
        registry.finish(&conv_id);
        emit_run_status(&app2, &conv_id, "idle");
    };
    // ... existing run_agent with cancel.as_ref() ...
    // on all returns: finish();
});
```

If `try_begin` fails, return `Err(TURN_IN_PROGRESS.to_string())` **without** emitting running.

- [ ] **Step 4: Compile check**

```bash
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox/src-tauri
cargo test --lib agent::run_registry
cargo check
```

Expected: compiles; registry tests still PASS.

---

### Task 4: Frontend `agentRuns` store

**Files:**
- Create: `src/stores/agentRuns.ts`
- Modify: `src/main.ts` only if Pinia stores need explicit registration (usually auto via `defineStore`)

**Interfaces:**
- Consumes: `TrajectoryStep` helpers `appendText` / `appendReasoning` from `@/utils/trajectory` (same ones HomePage uses today)
- Produces store `useAgentRunsStore`:
  - `runStatusById: Record<string, 'idle' | 'running'>`
  - `liveById: Record<string, TrajectoryStep[]>`
  - `streamMetaById: Record<string, 'live' | 'fallback' | null>` (optional; or fold into live)
  - `setStatus(id, status)`
  - `isRunning(id): boolean`
  - `ensureLive(id): TrajectoryStep[]` mutator helpers matching HomePage switch cases
  - `clearLive(id)`
  - `clearConversation(id)` — status idle + clear live (for delete)

- [ ] **Step 1: Implement store**

```typescript
import { defineStore } from 'pinia';
import {
  appendReasoning,
  appendText,
  type TrajectoryStep,
} from '@/utils/trajectory';

export type RunStatus = 'idle' | 'running';

export const useAgentRunsStore = defineStore('agentRuns', {
  state: () => ({
    runStatusById: {} as Record<string, RunStatus>,
    liveById: {} as Record<string, TrajectoryStep[]>,
    streamModeById: {} as Record<string, 'live' | 'fallback' | null>,
  }),
  getters: {
    isRunning: (state) => (id: string | null | undefined) =>
      !!id && state.runStatusById[id] === 'running',
    liveSteps: (state) => (id: string | null | undefined) =>
      (id && state.liveById[id]) || [],
  },
  actions: {
    setStatus(id: string, status: RunStatus) {
      this.runStatusById = { ...this.runStatusById, [id]: status };
      if (status === 'idle') {
        // keep live until clearLive after UI reload; caller clears
      }
    },
    applyAgentEvent(
      id: string,
      type: string,
      payload: {
        text?: string;
        id?: string;
        args?: unknown;
        result?: string;
        mode?: string;
      },
    ) {
      const steps = [...(this.liveById[id] ?? [])];
      switch (type) {
        case 'stream_meta':
          this.streamModeById = {
            ...this.streamModeById,
            [id]: payload.mode === 'live' ? 'live' : 'fallback',
          };
          break;
        case 'reasoning':
          this.liveById = {
            ...this.liveById,
            [id]: appendReasoning(steps, payload.text ?? ''),
          };
          break;
        case 'token':
          this.liveById = {
            ...this.liveById,
            [id]: appendText(steps, payload.text ?? ''),
          };
          break;
        case 'tool_start':
          this.liveById = {
            ...this.liveById,
            [id]: [
              ...steps,
              {
                type: 'tool',
                id: payload.id ?? 'tool',
                args: payload.args,
                status: 'running',
              },
            ],
          };
          break;
        case 'tool_end': {
          const next = [...steps];
          for (let i = next.length - 1; i >= 0; i--) {
            const s = next[i];
            if (s.type === 'tool' && s.id === payload.id && s.status === 'running') {
              next[i] = { ...s, result: payload.result, status: 'done' };
              break;
            }
          }
          this.liveById = { ...this.liveById, [id]: next };
          break;
        }
        default:
          break;
      }
    },
    clearLive(id: string) {
      const { [id]: _removed, ...rest } = this.liveById;
      this.liveById = rest;
      const { [id]: _m, ...modes } = this.streamModeById;
      this.streamModeById = modes;
    },
    clearConversation(id: string) {
      this.setStatus(id, 'idle');
      this.clearLive(id);
      const { [id]: _, ...rest } = this.runStatusById;
      this.runStatusById = rest;
    },
  },
});
```

Export `appendText` / `appendReasoning` from `trajectory.ts` if not already exported — check and export if missing.

- [ ] **Step 2: Sanity — Typecheck**

```bash
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox
npx vue-tsc --noEmit 2>&1 | head -40
```

Expected: no errors introduced by `agentRuns.ts` (fix export gaps first).

---

### Task 5: HomePage — route events, reconnect, cancel-by-id

**Files:**
- Modify: `src/views/HomePage.vue`

**Interfaces:**
- Consumes: `useAgentRunsStore`, `listen('agent-event')`, `listen('agent-run-status')`
- Produces: `turnBusy` derived from `agentRuns.isRunning(activeId)`; live UI from `agentRuns.liveSteps(activeId)`; cancel invokes with `conversationId`

- [ ] **Step 1: Remove early-drop of non-active events**

Delete this guard in the `agent-event` listener:

```typescript
if (convId && conversations.activeId && convId !== conversations.activeId) {
  return;
}
```

Replace listener body so **every** event with `convId` calls `agentRuns.applyAgentEvent(convId, type, p)` for stream types. For `error` / `interrupted` / `done`:

```typescript
case 'error':
  if (convId === conversations.activeId) {
    conversations.lastError = p.message ?? 'Agent 出错';
  }
  agentRuns.clearLive(convId);
  break;
case 'interrupted':
case 'done':
  agentRuns.clearLive(convId);
  if (convId === conversations.activeId) {
    // Prefer reload from DB (Rust already persisted):
    void conversations.openConversation(convId);
  }
  break;
```

Do **not** rely on local `finalizeStreaming` optimistic append for background ids; for active id, either keep a thin `finalizeStreaming` that only reloads, or always `openConversation` after clearLive (simpler, matches “DB is source of truth”).

Simplify: on `done`/`interrupted` for active id, call `openConversation`; remove duplicate local message append if reload covers it. If current UX flashes without local append, keep finalize **only when** `convId === activeId` **and** still prefer reload after.

- [ ] **Step 2: Listen `agent-run-status`**

```typescript
unlistenRunStatus = await listen<{ conversationId?: string; conversation_id?: string; status: string }>(
  'agent-run-status',
  (event) => {
    const p = event.payload;
    const id = p.conversationId ?? p.conversation_id;
    if (!id) return;
    const status = p.status === 'running' ? 'running' : 'idle';
    agentRuns.setStatus(id, status);
  },
);
```

Unregister on unmount.

- [ ] **Step 3: Derive busy + template binding**

```typescript
const agentRuns = useAgentRunsStore();
const turnBusy = computed(() => agentRuns.isRunning(conversations.activeId));
const activeTrajectory = computed(() => agentRuns.liveSteps(conversations.activeId));
```

Remove local `ref` duplicates for trajectory/busy where replaced. On `send`, do **not** set `turnBusy = true` manually — wait for status event; optionally optimistic:

```typescript
agentRuns.setStatus(conversationId, 'running');
agentRuns.clearLive(conversationId); // reset buffer
```

On send error including `turn_in_progress`:

```typescript
if (msg.includes('turn_in_progress')) {
  conversations.lastError = '该会话正在生成中，请稍候或先停止。';
  agentRuns.setStatus(conversationId, 'running'); // keep honest if race
}
```

- [ ] **Step 4: Cancel with id**

```typescript
const cancel = async () => {
  const id = conversations.activeId;
  if (!id) return;
  try {
    await invoke('cancel_chat_turn', { conversationId: id });
  } catch (error) {
    console.error('[chat] cancel failed:', error);
  }
};
```

- [ ] **Step 5: Manual smoke (dev)**

With `TBOX_AGENT_MOCK=1`: send on A, switch to B, send on B — both sidebar icons (Task 6) will show running; A’s stream still advances in store; return to A sees buffer.

---

### Task 6: SideBar leading icons

**Files:**
- Modify: `src/layout/SideBar.vue`

**Interfaces:**
- Consumes: `useAgentRunsStore().isRunning(item.id)`
- Produces: row `[icon][title]`

- [ ] **Step 1: Template**

Inside each `history-item`, before title/input:

```vue
<span class="history-status" aria-hidden="true">
  <i
    v-if="agentRuns.isRunning(item.id)"
    class="fas fa-spinner fa-spin history-status-running"
  ></i>
  <i v-else class="fas fa-message history-status-idle"></i>
</span>
```

Import store in script:

```typescript
import { useAgentRunsStore } from '@/stores/agentRuns';
const agentRuns = useAgentRunsStore();
```

- [ ] **Step 2: Styles**

```css
.history-status {
  flex: 0 0 14px;
  width: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
}

.history-status-idle {
  opacity: 0.45;
}

.history-status-running {
  color: var(--warning, #f59e0b);
  opacity: 1;
}
```

If `fa-message` missing in the bundled FA set, use `fa-comment`.

- [ ] **Step 3: Visual check**

Idle rows show muted message icon; running rows spin amber; active row title styles unchanged.

---

### Task 7: Delete-while-running + list cleanup

**Files:**
- Modify: `src/stores/conversations.ts` — `deleteConversation`
- Modify: `src/layout/SideBar.vue` — `deleteFromMenu` can stay if store handles cancel

**Interfaces:**
- Consumes: `invoke('cancel_chat_turn', { conversationId })`, `useAgentRunsStore().clearConversation`
- Produces: delete never leaves a stuck running icon

- [ ] **Step 1: Update `deleteConversation`**

```typescript
async deleteConversation(id: string) {
  this.lastError = null;
  try {
    const runs = useAgentRunsStore();
    if (runs.isRunning(id)) {
      try {
        await invoke('cancel_chat_turn', { conversationId: id });
      } catch {
        /* best-effort */
      }
    }
    await invoke('delete_conversation', { conversationId: id });
    runs.clearConversation(id);
    if (this.activeId === id) {
      this.newChat();
    }
    await this.loadList();
  } catch (error) {
    // existing error handling
  }
}
```

Avoid circular import issues: lazy-import inside action:

```typescript
const { useAgentRunsStore } = await import('@/stores/agentRuns');
```

or pass cancel from SideBar before delete — prefer store-owned sequence above.

- [ ] **Step 2: Verify**

Delete a mock-running conversation: cancel invoked, row gone, `runStatusById` has no entry.

---

### Task 8: End-to-end verification checklist

**Files:** none (manual)

- [ ] **Step 1: Run automated**

```bash
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox/src-tauri && cargo test --lib agent::run_registry
cd /Users/lh/Documents/dev/projects/hobby/rust/tbox && npx vue-tsc --noEmit
```

- [ ] **Step 2: Manual matrix (spec verification)**

1. A generating → switch to B → send on B: both sidebar rows running; neither cancels the other.
2. Stop on B: only B stops; A continues.
3. Return to A while running: see buffered text + continued streaming.
4. A completes: icon → message; live cleared; messages from DB.
5. Re-send on running A: `turn_in_progress` / Chinese error.
6. Delete running conversation: cancelled + removed; no stuck icon.
7. Right-click rename / trajectory / delete still work.

- [ ] **Step 3: Mark OpenSpec tasks complete**

Update `openspec/changes/parallel-conversation-runs/tasks.md` checkboxes to match done work. Do not archive until user requests `/opsx-archive`.

---

## Spec coverage self-check

| Spec requirement | Task |
|------------------|------|
| Approach B run registry | 2–3 |
| Parallel multi-conversation | 3, 5 |
| `cancel_chat_turn(conversationId)` | 3, 5 |
| `turn_in_progress` | 2–3, 5 |
| `agent-run-status` | 3, 5 |
| Sidebar two-state icons | 6 |
| Switch-away continue / switch-back reconnect | 4–5 |
| Cancel only when open | 5 (no sidebar stop) |
| Delete cancels first | 7 |
| OpenSpec before code | 1 |
| No error/unread badges / no SQLite run table | Global constraints |

## Placeholder / consistency self-check

- Error code fixed: `turn_in_progress` / `TURN_IN_PROGRESS`.
- Event name fixed: `agent-run-status`.
- Store name fixed: `agentRuns` / `useAgentRunsStore`.
- Icon classes fixed per spec.
- `Arc<RunRegistry>` manage pattern called out to avoid State-in-worker issues.
