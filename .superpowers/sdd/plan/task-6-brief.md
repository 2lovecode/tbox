# Task 3.1 — `/toolbox` 与对话空态壳

来源：plan.md Task 3.1；spec Conversation Homepage + tool-registry Navigate from toolbox

## Files

- Create: `src/views/ToolboxPage.vue` — 把 `HomePage.vue` 里的工具网格、分类过滤、搜索搬过来（保持原 class/交互）
- Modify: `src/views/HomePage.vue` — 改为对话空态（欢迎语 + 禁用或占位输入框即可，本任务不接 Agent）
- Modify: `src/layout/SideBar.vue` — 不再用分类列表当主内容。改为：新建对话按钮、历史占位（空列表即可）、底部「工具箱」入口 `goToolbox()` → `/toolbox`
- Modify: `src/router/main.ts`：增加 `{ path: '/toolbox', component: ToolboxPage }`；`/` 仍 HomePage
- `SideBar.vue` 里 `openCategory` 当前 `router.push('/')` — 分类若仍展示，必须去 `/toolbox`。本任务侧栏主结构按 ChatGPT 式，分类留在工具箱页。

Spotlight 工具导航保持原工具路由，不要改成 `/`。

## Verify

`pnpm exec vue-tsc --noEmit`

勾选 tasks.md 3.1

## Commit

只 add 上述 vue/ts + tasks.md。Never `git add .`

`git commit -m "feat: move tool grid to /toolbox and show chat shell on home"`
