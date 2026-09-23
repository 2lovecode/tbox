# Chat & Shell Visual Unification — Design

Date: 2026-09-22  
Status: approved (Approach A / implementation approach 1)  
Scope: display / tokens / settings chrome only — no OpenSpec product-behavior change

## Goal

1. JSON in chat/trajectory defaults to **compact** display (shorter vertical footprint) unless explicitly prettified.
2. Code/JSON blocks follow the **current theme** (no hard-coded near-black card); contrast softer than today.
3. **Dark mode** palette reconsidered toward neutral greys (less blue-purple).
4. **Settings + chat + shared chrome** share the same shell tokens; tool pages only inherit CSS variables (no per-tool layout rewrite).

## Confirmed decisions

| Topic | Choice |
|-------|--------|
| Overall scope | **A** — tokens + chat / settings / code blocks / dark mode; tools inherit vars only |
| Implementation approach | **1** — extend `App.vue` tokens; compact JSON in `markdown.ts` + trajectory args; flatten settings chrome |
| JSON default | Compact (`JSON.stringify` without indent) when content is valid JSON and not already an explicit pretty form |
| Explicit pretty kept when | Fenced block already indented; or result from explicit format tools (e.g. `json.format`) that already ship pretty text |
| Copy behavior | Clipboard = what is shown (compact if compact) |
| Code block surface | Theme tokens (`--code-bg` / `--code-fg` / shell divider), not `#1e1e2e` |
| Dark palette | Neutral greys; keep readable text; soft primary |
| Settings | Flat plane + `--shell-divider` / `--shell-gutter` / `--control-radius`; no double card nesting |
| Out of scope | Rewriting 40+ tool page layouts; OpenSpec capability changes |

## Approach detail

### Tokens (`App.vue` `:root` + `.dark-mode`)

Add / adjust:

- `--code-bg`, `--code-fg`, `--code-muted`, `--code-border` (derived from bg/text/shell)
- Optional semantic: `--warning`, `--danger`, `--success` for banners/errors (replace hard-coded dark-mode hex where cheap)
- Dark mode surfaces, e.g.:
  - `--bg-primary: #12141a`
  - `--bg-secondary: #181b22`
  - `--bg-tertiary: #22262f`
  - text/border remain accessible; `--primary` slightly desaturated if needed

Light mode keeps current plane; code blocks use the same token names with light mixes.

### Markdown / JSON (`src/utils/markdown.ts`)

In `highlight` (and/or `wrapPlainStructured` prep):

- If language is `json` (or bare JSON wrapped as json) **and** parse succeeds:
  - If source already looks pretty (contains newlines + typical indent) **and** was user/model-supplied fence → keep as-is
  - Else re-serialize with `JSON.stringify(parsed)` (no space) for display
- Non-JSON languages unchanged
- Keep hljs + copy button wiring

Heuristic for “already pretty”: e.g. trimmed text includes `\n` and a line starting with two+ spaces or tab after `{`/`[`. Prefer preserving intentional pretty dumps from format tools.

### Trajectory args (`AssistantTrajectory.vue`)

- `formatArgs`: default `JSON.stringify(args)` compact; word-wrap via CSS already on `pre`

### Code block CSS (`App.vue` global `.md-content .md-code-block`)

- Background/border/color from `--code-*`
- Soften copy-button styles to match (no white-on-black assumption)
- Tune or override `atom-one-dark` token colors so they sit on soft `--code-bg` (light + dark); prefer CSS variable overrides over swapping theme mid-flight unless one-dark remains too loud

### Settings (`SettingsPage.vue` + panels if needed)

- Align padding with `--shell-gutter`
- Nav / content: remove heavy bordered “card stack” look where it fights the shell; use divider + flat bg (`var(--bg-primary)` / transparent) consistent with sidebar/chat
- Controls: `--control-radius`, shared hover/active (primary tint already used — keep, but on flat surface)
- Do **not** change settings routes or section model

### HomePage dark overrides

- Replace one-off `:global(.dark-mode) … #fbbf24` etc. with semantic tokens where straightforward

## Out of scope

- Per-tool page layout unification
- Changing JSON tool’s own “format pretty” product behavior
- Locale / i18n expansion beyond existing trajectory strings
- Persist theme tokens to disk beyond existing dark-mode toggle

## Verification

- [ ] Chat: unsolicited JSON shows compact; vertical height clearly shorter than pretty
- [ ] Chat: explicitly pretty JSON (or format-tool pretty) still readable multi-line
- [ ] Light + dark: code blocks are a mild lift over page bg, not a black island
- [ ] Dark mode: greys, no strong blue-purple panels; text contrast OK
- [ ] Settings: same shell language as chat (dividers, gutter, flat); nav still works
- [ ] Sample tool page: colors follow new dark tokens without layout regression
- [ ] Copy button on code block still works

## Files to touch (expected)

1. `src/App.vue` — tokens, `.md-code-block`, dark palette
2. `src/utils/markdown.ts` — compact JSON display path
3. `src/components/AssistantTrajectory.vue` — compact `formatArgs`
4. `src/views/SettingsPage.vue` — flatten chrome to shell tokens
5. `src/views/HomePage.vue` — dark semantic token cleanup (light touch)
6. `src/main.ts` — only if highlight theme import changes

## Non-goals reminder

Tool pages inherit variables automatically; no systematic rewrite of `src/views/tools/*` in this change.
