# Chat Layout Density Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enlarge the chat surface by tightening global chrome and switching the shell to a flat plane divided by thin lines.

**Architecture:** Pure CSS/layout changes in the App shell, SideBar, and HomePage. No Vue logic, routing, or agent behavior changes except adjusting `fitHeight()` margin constants if body padding/gap shrink.

**Tech Stack:** Vue 3 SFC scoped + global styles in `App.vue`; existing CSS variables (`--bg-primary`, `--border-color`, etc.).

## Global Constraints

- Scope is visual/layout only — no OpenSpec change required.
- Do not change message bubbles, trajectory internals, or markdown styles beyond what inherits from the flatter shell.
- Light and dark modes must both stay flat + line-divided.
- Message list remains the only scroll container; window must not grow a document scrollbar.
- Do not commit unless the user explicitly asks.

---

## File map

| File | Responsibility |
|------|----------------|
| `src/App.vue` | Body padding/bg, container grid, header, footer, divider lines |
| `src/layout/SideBar.vue` | Sidebar width density, remove card chrome, flat tokens |
| `src/views/HomePage.vue` | Chat full-width, flatten message-list, tweak `fitHeight()` |

---

### Task 1: Flatten and densify App shell

**Files:**
- Modify: `src/App.vue` (global `<style>` block: `body`, `.container`, `header`, logo, actions, footer, dark-mode)

**Interfaces:**
- Consumes: existing CSS vars `--bg-primary`, `--bg-secondary`, `--border-color`, `--text-primary`, `--shadow`
- Produces: full-bleed flat shell; grid column for sidebar expected at `176px` (Task 2 styles the aside to match)

- [x] **Step 1: Update body and container**

In `src/App.vue` global styles, replace:

```css
body {
  background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-tertiary) 100%);
  color: var(--text-primary);
  height: 100%;
  overflow: hidden;
  padding: 16px 20px;
  transition: background 0.3s ease, color 0.3s ease;
}

.container {
  max-width: 1400px;
  margin: 0 auto;
  height: 100%;
  max-height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: 210px 1fr;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 16px 20px;
  overflow: hidden;
}
```

with:

```css
body {
  background: var(--bg-primary);
  color: var(--text-primary);
  height: 100%;
  overflow: hidden;
  padding: 0;
  transition: background 0.3s ease, color 0.3s ease;
}

.container {
  max-width: none;
  width: 100%;
  margin: 0;
  height: 100%;
  max-height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: 176px 1fr;
  grid-template-rows: auto minmax(0, 1fr) auto;
  gap: 0;
  overflow: hidden;
}
```

- [x] **Step 2: Compact header and footer; add line dividers**

Replace header / logo / actions / footer rules so they match this density:

```css
header {
  grid-column: 1 / -1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 14px;
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
  background: var(--bg-primary);
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  transition: var(--transition);
}

.logo:hover {
  transform: none;
  opacity: 0.9;
}

.logo-icon {
  width: 32px;
  height: 32px;
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 14px;
  box-shadow: none;
}

.logo-text {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.theme-toggle {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-primary);
  font-size: 14px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: var(--transition);
  box-shadow: none;
}

.theme-toggle:hover {
  transform: none;
  background: color-mix(in srgb, var(--primary) 10%, transparent);
  box-shadow: none;
}

.spotlight-trigger {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-width: 180px;
  padding: 6px 12px;
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  font-family: inherit;
  font-size: 13px;
  box-shadow: none;
  transition: var(--transition);
}

.spotlight-trigger:hover {
  border-color: var(--primary);
  color: var(--primary);
  transform: none;
  box-shadow: none;
}

.spotlight-trigger-kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 40px;
  height: 22px;
  padding: 0 8px;
  border-radius: 6px;
  background: color-mix(in srgb, var(--bg-secondary) 80%, transparent);
  border: 1px solid var(--border-color);
  color: var(--text-primary);
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

footer {
  grid-column: 1 / -1;
  text-align: center;
  padding: 6px 14px;
  color: var(--text-secondary);
  font-size: 12px;
  border-top: 1px solid var(--border-color);
  margin-top: 0;
  flex-shrink: 0;
  background: var(--bg-primary);
}
```

Also set `.loading-container` background to `var(--bg-primary)` and remove its heavy card shadow if present (optional: keep a light border).

- [x] **Step 3: Visual check shell only**

Run the Tauri/Vite app (or open existing dev session). On `/` and `/toolbox`:
- No body gradient; page is one flat color
- Header and footer are short; thin divider lines visible
- No large outer margin around the window content

- [ ] **Step 4: Commit only if user asks**

Skip unless explicitly requested.

---

### Task 2: Flatten SideBar to match shell

**Files:**
- Modify: `src/layout/SideBar.vue` (`<style scoped>`)

**Interfaces:**
- Consumes: App grid column `176px` from Task 1
- Produces: flat sidebar with right border; no card fill/shadow

- [x] **Step 1: Replace `.sidebar` card chrome**

Replace `.sidebar` block with:

```css
.sidebar {
  background: var(--bg-primary);
  border-radius: 0;
  padding: 10px 8px;
  box-shadow: none;
  border-right: 1px solid var(--border-color);
  height: 100%;
  min-height: 0;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow: hidden;
  align-self: stretch;
}
```

- [x] **Step 2: Tighten controls and history rows**

Update these rules (keep behavior/classes unchanged):

```css
.new-chat-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 8px 10px;
  border: 1px dashed color-mix(in srgb, var(--primary) 35%, transparent);
  background: color-mix(in srgb, var(--primary) 6%, transparent);
  color: var(--primary);
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.history-section h3 {
  margin: 0 0 8px;
  font-size: 11px;
  color: var(--text-secondary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.history-empty {
  padding: 16px 10px;
  text-align: center;
  color: var(--text-secondary);
  background: transparent;
  border: 1px dashed color-mix(in srgb, var(--border-color) 80%, transparent);
  border-radius: 8px;
}

.history-empty p {
  margin: 0 0 6px;
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 8px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: background 0.15s ease;
}

.history-item:hover {
  background: color-mix(in srgb, var(--text-secondary) 10%, transparent);
}

.history-item.active {
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  color: var(--primary);
  font-weight: 600;
}

.history-delete {
  flex-shrink: 0;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease, color 0.15s ease, background 0.15s ease;
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid var(--border-color);
}

.toolbox-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.toolbox-btn:hover {
  background: color-mix(in srgb, var(--text-secondary) 10%, transparent);
  color: var(--primary);
}

.toolbox-btn.active {
  background: color-mix(in srgb, var(--primary) 12%, transparent);
  color: var(--primary);
  font-weight: 600;
}
```

Also update hardcoded light-only colors in remaining sidebar rules (`#475569`, `#f1f5f9`, etc.) to the token-based versions above so dark mode stays coherent.

- [x] **Step 3: Visual check sidebar**

On `/`: sidebar shares page bg; only a right divider separates it; rows feel denser; dark mode readable.

- [ ] **Step 4: Commit only if user asks**

---

### Task 3: Expand chat plane and remove message-list card

**Files:**
- Modify: `src/views/HomePage.vue` (`.chat-home`, `.message-list`, `.composer` shadow if needed, `fitHeight()`)

**Interfaces:**
- Consumes: full main-wrapper width from Tasks 1–2
- Produces: chat content nearly full-bleed within main column (~16–24px inset)

- [x] **Step 1: Widen `.chat-home` and flatten `.message-list`**

Replace:

```css
.chat-home {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  max-width: 860px;
  margin: 0 auto;
  padding: 20px 12px 12px;
  /* ... */
}

.message-list {
  /* ... */
  padding: 16px 16px 20px;
  border-radius: 16px;
  background: color-mix(in srgb, var(--bg-primary, #fff) 82%, var(--bg-tertiary, #e4edf5));
  border: 1px solid color-mix(in srgb, var(--border-color, rgba(0, 0, 0, 0.1)) 55%, transparent);
  box-shadow: inset 0 1px 0 color-mix(in srgb, var(--bg-primary, #fff) 70%, transparent);
}
```

with:

```css
.chat-home {
  display: flex;
  flex-direction: column;
  gap: 0;
  width: 100%;
  max-width: none;
  margin: 0;
  padding: 8px 20px 10px;
  overflow: hidden;
  height: 100%;
  max-height: 100%;
  box-sizing: border-box;
  position: relative;
}

.message-list {
  flex: 1;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 18px;
  padding: 8px 4px 16px;
  scrollbar-gutter: stable;
  border-radius: 0;
  background: transparent;
  border: none;
  box-shadow: none;
}
```

- [x] **Step 2: Soften composer to sit on the flat plane**

Replace `.composer` shadow-heavy card with a top divider + light border:

```css
.composer {
  flex-shrink: 0;
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding: 10px 12px;
  margin-top: 0;
  background: var(--bg-primary);
  border-radius: 12px;
  box-shadow: none;
  border: 1px solid var(--border-color);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.composer:focus-within {
  border-color: color-mix(in srgb, var(--primary) 45%, var(--border-color));
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
}
```

- [x] **Step 3: Fix `fitHeight()` for zero outer chrome padding**

In `fitHeight()` inside `HomePage.vue`, change:

```ts
const footerH = footer ? footer.getBoundingClientRect().height + 20 : 0;
const height = Math.max(window.innerHeight - top - footerH - 20, 320);
```

to:

```ts
const footerH = footer ? footer.getBoundingClientRect().height : 0;
const height = Math.max(window.innerHeight - top - footerH, 320);
```

(Because body padding and container gap are now 0; the previous `+20` / `-20` fudge was for the old margins.)

- [x] **Step 4: End-to-end visual verification**

Checklist:
1. Chat page: messages nearly full main width; little unused horizontal space
2. No nested gray/white panels in message list
3. Header / sidebar / footer divided by lines only
4. Light + dark both coherent
5. Narrow width (<900px): sidebar still hides via existing media query
6. Sending/streaming: message list still scrolls; no window scrollbar
7. `/toolbox` still usable with denser chrome

- [ ] **Step 5: Commit only if user asks**

---

## Spec coverage self-check

| Spec requirement | Task |
|------------------|------|
| Global chrome tighten | Task 1 |
| Sidebar denser + flat | Task 2 |
| Chat nearly full width 16–24px inset | Task 3 Step 1 |
| Flat bg + line dividers | Tasks 1–2 |
| Remove message-list card bg | Task 3 Step 1 |
| fitHeight after padding removal | Task 3 Step 3 |
| Out of scope (bubbles/trajectory/logic) | untouched |
