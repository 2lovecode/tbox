# Chat Layout Density — Design

Date: 2026-09-22  
Status: approved for implementation (pending user review of this doc)  
Scope: visual / layout only (no OpenSpec change; no observable product behavior change)

## Goal

Enlarge the chat reading/writing surface by shrinking shared chrome and removing card-like background nesting, so the UI reads as one flat plane divided by thin lines.

## Decisions (confirmed)

| Topic | Choice |
|-------|--------|
| Chrome scope | Global (chat, toolbox, and any page using the App shell) |
| Chat width | Nearly fill main column; ~16–24px horizontal inset only |
| Region separation | Single flat background; thin divider lines only (no tint blocks) |

## Approach

Shell density compression (Approach 1): tighten `App.vue` / `SideBar.vue` / `HomePage.vue` styles. No routing or agent logic changes.

## Layout targets

### Global shell (`App.vue`)

- `body` padding: `16px 20px` → `0` (or at most `4–8px` if needed for edge safety)
- `.container`: remove / raise `max-width: 1400px` so content uses full window; reduce `gap` (e.g. `16px 20px` → `0`)
- Header: compact height
  - logo icon ~48px → ~32–36px
  - logo text ~28px → ~18–20px
  - theme/settings buttons ~45px → ~32px
  - Spotlight trigger: flatter padding, shorter min-width
- Footer: smaller padding and font (~12px); keep copyright line
- Background: drop body gradient; use flat `--bg-primary` (light + dark)
- Dividers: header bottom, footer top, sidebar right → `1px solid var(--border-color)` (optionally softened)

### Sidebar (`SideBar.vue`)

- Grid column: `210px` → `168–180px`
- Remove card look: no white panel fill distinct from page, no shadow, no large radius
- Tighten padding, new-chat button, history row height
- Align empty-state / hover backgrounds to flat tokens (hover may keep a light highlight; not a separate panel)

### Chat home (`HomePage.vue`)

- `.chat-home`: remove `max-width: 860px`; width `100%`; horizontal padding ~16–24px; reduce top padding
- `.message-list`: remove card background, inset shadow, and large radius; same plane as page; optional none/borderless
- Composer: keep as the primary interactive surface (border OK); avoid heavy stacked shadows if they fight the flat shell — light border is enough
- Adjust `fitHeight()` footer/margin fudge if body padding/gap change so the window still has no outer scrollbar

## Out of scope

- Message bubble / trajectory / markdown internals
- Agent, streaming, conversation store behavior
- Collapsible sidebar or hiding the footer
- Tool page content layouts beyond inheriting the tighter shell

## Verification

- Chat home: messages use nearly full main column; little unused margin
- Toolbox / settings / tool pages: chrome denser but usable; no broken grid
- Light + dark: flat bg + line dividers consistent
- Resize / narrow (<900px): existing sidebar hide still works
- No document-level scrollbar; message list remains the scroll container

## Files to touch

1. `src/App.vue` — shell grid, header, footer, body bg
2. `src/layout/SideBar.vue` — width/density, flat style
3. `src/views/HomePage.vue` — chat width, message-list plane, fitHeight constants if needed
