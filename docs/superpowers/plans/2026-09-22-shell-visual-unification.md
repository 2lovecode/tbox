# Shell Visual Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compact JSON in chat by default, theme-matched soft code blocks, neutral dark palette, and settings chrome aligned with the shell — without rewriting tool pages.

**Architecture:** Extend CSS design tokens in `App.vue`; compact JSON in `markdown.ts` highlight path; compact trajectory `formatArgs`; flatten `SettingsPage` to shell dividers/gutters; light HomePage dark-mode token cleanup.

**Tech Stack:** Vue 3, markdown-it, highlight.js, existing App shell CSS variables.

## Global Constraints

- Scope A only: tokens + chat/settings/code/dark; tool pages inherit vars, no layout rewrite.
- No OpenSpec product-behavior change; display-layer only.
- Do not create git commits unless the user explicitly asks.
- Chinese UI copy unchanged except where already localized.

---

## File map

| File | Responsibility |
|------|----------------|
| `src/utils/jsonDisplay.ts` | Pure helpers: detect pretty JSON, compact for display |
| `src/utils/markdown.ts` | Use helpers when highlighting/wrapping `json` |
| `src/components/AssistantTrajectory.vue` | Compact `formatArgs` |
| `src/App.vue` | Dark greys, `--code-*`, `--warning`/`--danger`/`--success`, code-block CSS |
| `src/main.ts` | Only if highlight theme import must change |
| `src/views/SettingsPage.vue` | Flat shell-aligned chrome |
| `src/views/HomePage.vue` | Replace hard-coded dark hex with semantic tokens |

---

### Task 1: Compact JSON display helper + markdown wiring

**Files:**
- Create: `src/utils/jsonDisplay.ts`
- Modify: `src/utils/markdown.ts`
- Test: Vitest if present; else manual via Node assert script / `pnpm` unit if repo has none — prefer exporting pure functions and verifying with a one-off `node --experimental-strip-types` or existing test runner

**Interfaces:**
- Produces:
  - `looksPrettyJson(raw: string): boolean`
  - `compactJsonForDisplay(raw: string): string` — returns compact stringify if parseable and not pretty; else original
  - `prepareJsonFenceBody(raw: string): string` — for fence body before highlight

- [ ] **Step 1: Add `src/utils/jsonDisplay.ts`**

```ts
/** Heuristic: multi-line JSON with indented lines after opener. */
export function looksPrettyJson(raw: string): boolean {
  const t = raw.trim();
  if (!t.includes('\n')) return false;
  return /\n[ \t]{2,}["{\[]/.test(t) || /\n[ \t]+}/.test(t);
}

/** If valid JSON and not already pretty, return compact stringify; else original. */
export function compactJsonForDisplay(raw: string): string {
  const t = raw.trim();
  if (!t) return raw;
  try {
    const parsed = JSON.parse(t);
    if (looksPrettyJson(t)) return raw;
    return JSON.stringify(parsed);
  } catch {
    return raw;
  }
}

export function prepareJsonFenceBody(raw: string): string {
  return compactJsonForDisplay(raw);
}
```

- [ ] **Step 2: Wire `markdown.ts` highlight + wrap**

In `highlight(code, rawLang)`: when `resolveLang(rawLang) === 'json'` or rawLang trim is `json`, set `code = prepareJsonFenceBody(code)` before hljs.

In `wrapPlainStructured`: after `JSON.parse` succeeds, wrap with compact body:

```ts
const body = looksPrettyJson(t) ? t : JSON.stringify(JSON.parse(t));
return `\`\`\`json\n${body}\n\`\`\``;
```

(Reuse `compactJsonForDisplay` / `looksPrettyJson` from `jsonDisplay.ts`.)

- [ ] **Step 3: Sanity-check helper**

Run (from repo root):

```bash
npx --yes tsx -e "
import { compactJsonForDisplay, looksPrettyJson } from './src/utils/jsonDisplay.ts';
const pretty = '{\n  \"a\": 1\n}';
const mini = '{\"a\":1}';
console.assert(looksPrettyJson(pretty) === true);
console.assert(compactJsonForDisplay(mini) === '{\"a\":1}');
console.assert(compactJsonForDisplay(pretty) === pretty);
console.assert(compactJsonForDisplay('{\"b\":2,\"a\":1}') === '{\"b\":2,\"a\":1}');
console.log('ok');
"
```

Expected: `ok`

- [ ] **Step 4: Compact trajectory args**

In `AssistantTrajectory.vue` `formatArgs`:

```ts
function formatArgs(args: unknown): string {
  try {
    return JSON.stringify(args) ?? '';
  } catch {
    return String(args);
  }
}
```

---

### Task 2: Theme tokens + soft code blocks + dark palette

**Files:**
- Modify: `src/App.vue` (`:root`, `.dark-mode`, `.md-content .md-code-block*` styles)
- Modify: `src/main.ts` only if needed
- Modify: `src/views/HomePage.vue` dark hard-coded colors

**Interfaces:**
- Consumes: none from Task 1
- Produces CSS vars: `--code-bg`, `--code-fg`, `--code-muted`, `--code-border`, `--warning`, `--danger`, `--success`

- [ ] **Step 1: Extend `:root` tokens**

```css
:root {
  /* existing vars… */
  --code-bg: color-mix(in srgb, var(--bg-tertiary) 42%, var(--bg-primary));
  --code-fg: var(--text-primary);
  --code-muted: var(--text-secondary);
  --code-border: var(--shell-divider);
  --warning: #b45309;
  --danger: #dc2626;
  --success: #16a34a;
}
```

- [ ] **Step 2: Neutral dark palette**

```css
.dark-mode {
  --bg-primary: #12141a;
  --bg-secondary: #181b22;
  --bg-tertiary: #22262f;
  --text-primary: #e4e4e7;
  --text-secondary: #a1a1aa;
  --border-color: rgba(255, 255, 255, 0.09);
  --shadow: 0 4px 20px rgba(0, 0, 0, 0.35);
  --shell-divider: color-mix(in srgb, var(--border-color) 80%, transparent);
  --code-bg: color-mix(in srgb, var(--bg-tertiary) 70%, var(--bg-primary));
  --code-fg: #e4e4e7;
  --code-muted: #9ca3af;
  --code-border: var(--shell-divider);
  --warning: #fbbf24;
  --danger: #f87171;
  --success: #4ade80;
  --primary: #5b7cfa; /* slightly softer if current feels loud */
}
```

Keep `--primary` tweak optional; if chat accents break, leave `--primary` as `#4361ee`.

- [ ] **Step 3: Retheme `.md-code-block`**

Replace hard-coded `#1e1e2e` / `#cdd6f4` / `#7f8496` / rgba white copy btn with:

```css
.md-content .md-code-block {
  background: var(--code-bg);
  border: 1px solid var(--code-border);
  /* keep position/padding/radius */
}
.md-content .md-code-block code {
  color: var(--code-fg);
}
.md-content .md-code-lang {
  color: var(--code-muted);
}
.md-content .md-code-copy {
  background: color-mix(in srgb, var(--text-secondary) 12%, transparent);
  color: var(--code-muted);
}
.md-content .md-code-copy:hover,
.md-content .md-code-copy:focus-visible {
  background: color-mix(in srgb, var(--text-secondary) 22%, transparent);
  color: var(--code-fg);
}
.md-content .md-code-copy.copied {
  color: var(--success);
}
```

Add light overrides for hljs if atom-one-dark forces bright-on-dark that fights light `--code-bg`:

```css
.md-content .md-code-block .hljs {
  background: transparent;
  color: var(--code-fg);
}
```

If light mode keyword colors from atom-one-dark are unreadable, switch `main.ts` import to `highlight.js/styles/github.css` and keep dark via:

```css
.dark-mode .md-content .md-code-block .hljs-/* … */
```

Prefer minimal change: keep `atom-one-dark` first; only swap if light mode looks broken.

- [ ] **Step 4: HomePage semantic colors**

Replace e.g.:

```css
:global(.dark-mode) .llm-banner strong { color: #fbbf24; }
```

with `color: var(--warning);` (and danger/success similarly). Ensure vars exist on `.container.dark-mode` ancestors (they do via App).

---

### Task 3: Settings page shell alignment

**Files:**
- Modify: `src/views/SettingsPage.vue` scoped styles

**Interfaces:**
- Consumes: `--shell-gutter`, `--shell-divider`, `--control-radius`, `--bg-*`, `--text-*`

- [ ] **Step 1: Flatten settings chrome**

Update styles conceptually:

```css
.settings-page {
  gap: 12px;
  max-width: none;
  margin: 0;
  padding: var(--shell-gutter);
  /* keep height/overflow */
}

.settings-layout {
  grid-template-columns: 200px minmax(0, 1fr);
  gap: var(--shell-gutter);
}

.settings-nav {
  border: none;
  border-right: 1px solid var(--shell-divider);
  border-radius: 0;
  background: transparent;
  padding: 0 12px 0 0;
  position: static;
}

.settings-content {
  border: none;
  border-radius: 0;
  background: transparent;
  padding: 0 0 0 4px;
  overflow: auto;
}

.back-btn,
.nav-item {
  border-radius: var(--control-radius);
}

.nav-item.active {
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  color: var(--primary);
}
```

Preserve markup/structure and section routing.

- [ ] **Step 2: Visual check**

Run app (`pnpm tauri dev` or existing frontend dev). Verify settings + chat light/dark: no black code island; compact JSON; settings flat.

---

## Spec coverage self-check

| Spec requirement | Task |
|------------------|------|
| Compact JSON default | Task 1 |
| Keep explicit pretty | Task 1 (`looksPrettyJson`) |
| Theme code bg | Task 2 |
| Soft contrast | Task 2 |
| Neutral dark | Task 2 |
| Settings unified | Task 3 |
| Tools inherit only | (no tool files touched) |

## Placeholder scan

None intentional.

---

## Execution

User already approved implementation. Prefer **inline execution** in this session (executing-plans), skipping commit steps unless requested.
