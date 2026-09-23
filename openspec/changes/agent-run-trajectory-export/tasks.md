## 1. 导出工具函数

- [x] 1.1 新增 `src/utils/agentRunExport.ts`：导出 `buildAgentRunMarkdown(run, messages)` 与 `buildAgentRunJson(run, messages)` 两个纯函数，输入与 `get_conversation_messages` 返回结构对齐（参数命名上把 conversation 称作 run），输出按 spec 中「Agent Run Trajectory Export」规定的 Markdown / JSON 形态：Markdown 里助手回合标题为 `## Assistant (Turn N/M)`、列表行展示 trajectory 数（即助手回合数）；JSON 顶层包成 `{ run: { ... }, messages: [...] }`。文件顶部加注释对齐 `src-tauri/src/agent/trajectory.rs` 的 `TrajectoryStep` 字段。验证：`npx vue-tsc --noEmit` 不报类型错误。

- [x] 1.2 复用 `downloadTextFile`（`@/utils/download`）与项目既有 `useClipboard.ts`，在 `agentRunExport.ts` 内再导出 `exportAgentRunAs(conv, messages, 'md' | 'json')` 与 `copyAgentRunAsMarkdown(conv, messages)`，封装文件名拼接（`<sanitized-title>-<id前8>-<YYYYMMDD>.ext`）与 MIME 选择（`text/markdown;charset=utf-8` / `application/json;charset=utf-8`）。验证：手动浏览器 console 触发一次能拿到文件名形如 `my-chat-3f2c1a90-20260921.md`。

## 2. Agent 运行页面（列表 + 详情合并 Vue）

- [x] 2.1 新增 `src/views/AgentRuns.vue`：script setup 内根据 `route.params.id` 切换「列表模式 / 详情模式」；列表模式调用 `useConversationsStore().loadList()`，每行展示标题、updated_at（本地时区）、消息数、**Trajectory 数**（=该 Run 内助手回合数，渲染时即时计算或基于消息长度缓存）、是否含工具调用（取任一助手消息的 `tool_calls_json` / `trajectory_json` 非空判断）、是否压缩（读 `context_budget` 字段）、是否中断（消息文本含中断标识）；空态按 spec 给出引导文案。验证：`npm run dev` 进入 `/agent-runs` 能看到列表与 trajectory 列，空库时显示空态。

- [x] 2.2 详情模式：进入时若 `messages` 为空则 `await conversations.openConversation(id)`；按消息顺序渲染用户消息（纯文本）+ 助手回合（`<AssistantTrajectory :steps="messageSteps(msg)" :streaming="false" :stream-mode="null" />`），**每个 trajectory 节点默认折叠**（reasoning/tool 收起、text 展开，沿用 AssistantTrajectory 默认 `streaming=false` 行为；不写自定义强制展开覆盖）；**每个助手回合上方标注 `Turn N/M`（N = 当前回合序号、M = 该 Run 总助手回合数）**；会话元信息放在顶部（标题、id、时间范围、消息总数、Trajectory 数、context_budget 摘要）；右上角放「导出 Markdown / 导出 JSON / 复制 Markdown」三按钮，按 spec 在空 Run 时禁用。验证：手动跑一个含多个工具调用回合的会话，从 `/agent-runs` 点入能看到 reasoning/tool 默认收起、text 可见、回合序号正确。

- [x] 2.3 详情页顶部加「返回列表」与「返回聊天」两个文字按钮，分别 `router.push('/agent-runs')` 与 `router.push('/')` + `conversations.openConversation(id)`；保持与现有 `App.vue` 一致的 sidebar 显隐规则（已扩展至 `/agent-runs*`）。验证：在详情页点返回能跳到列表或聊天首页。

## 3. 路由 + 聊天首页会话内入口 + 侧边栏次入口

- [x] 3.1 `src/router/main.ts` 新增两条路由（动态 import）：`/agent-runs` 与 `/agent-runs/:id`（id 可选，单文件共用）。验证：`/agent-runs` 渲染列表，`/agent-runs/<某 id>` 渲染详情；未知 id 不崩，详情页显示「会话不存在」提示。

- [x] 3.2 `src/layout/SideBar.vue` 在历史列表与「工具箱」入口之间加一个「Agent 运行」入口（次入口），路由跳转 `/agent-runs`，沿用现有 SideBar 视觉规范（图标 + 文字，激活态高亮 `/agent-runs*`）。验证：在任意页面点该入口都能跳转到 `/agent-runs`。

- [x] 3.3 **聊天首页会话内主入口**：`src/views/HomePage.vue` 在消息列表上方（错误条之下、消息列表之上）的元信息区新增「查看完整轨迹」按钮/链接，点击 `router.push('/agent-runs/:id')`；**仅当 `conversations.activeId` 非空且消息数 ≥ 1 时显示**。视觉上与该区域现有 context budget 环/模型切换器保持同一轻量风格。验证：发送第一条消息后该入口出现并可跳；新建空草稿（未发消息）入口不显示。

## 4. 删除级联（契约性声明，无代码变更）

- [x] 4.1 验证既有 FK `FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE`（`src-tauri/src/commands/conversation.rs:47`）确实生效：阅读 SCHEMA_SQL 与 delete_conversation_on 即可确认，无需写新 Rust。手动回归：删除某个会话后，`/agent-runs` 列表中该 Run 消失；访问已被删除的 `/agent-runs/:id` 显示「会话不存在」空态，不暴露残留轨迹。

## 5. 检查与归档

- [x] 5.1 跑 `npx vue-tsc --noEmit`、`npm run build`，确认无类型 / 构建错误。验证：CI 视角下与 main 无差异以外的报错。

- [x] 5.2 手动验证四条核心场景：① 聊天首页发送消息后「查看完整轨迹」出现并跳转；② Markdown 导出文件能被 VSCode 正常渲染（含推理引用、工具代码块、用户/助手分段）；③ JSON 导出能被 `jq .` 正常解析，且不含敏感字段；④ 删除会话后列表与详情均不残留。

- [x] 5.3 跑 `openspec validate --changes` 与 `openspec validate --specs`，无错。验证：两条命令均通过。

- [ ] 5.4 提交后用 `/opsx-archive`（或 `openspec archive --change agent-run-trajectory-export`）归档，把 delta spec 同步进 `openspec/specs/agent-chat/spec.md`。验证：`openspec list --specs` 中 `agent-chat` 的 requirements 数 +2。

## 6. DeepSeek Harness 账本对齐（详情页重做）

- [x] 6.1 新增 `src/utils/sessionTrajectory.ts`：把 `session_events`（空则回退 messages/trajectory_json）投影为 Turn → 消息/第 M 步 → 紧凑记录（kind: user/message/tool/context/compacted）。工具 call/result FIFO 配对；耗时由事件时间差得出。验证：投影函数对「一轮用户 + 一步工具 + 一步文本」产出两步分组。

- [x] 6.2 重写 `AgentRunTrajectory.vue` 为 DSH 风格账本：`#N` + kind 色标 + 单行摘要 + 耗时；工具栏（收起轮次 / 收起调用 / 搜索）；点击行打开右侧检查器。验证：`npx vue-tsc --noEmit` 通过。

- [x] 6.3 `AgentRuns.vue` 详情改用账本组件；导出 Markdown 按「第 N 轮 / 消息 / 第 M 步 / #N 用户|助手|工具」组织。验证：类型检查通过。

- [x] 6.4 工具步 `assistant/message` 写入该步思考与完整 tool_calls（含参数）；账本展示 `request/header` 为系统行；无思考的旧日志从 `trajectory_json` 回填。验证：`cargo test` 覆盖工具步 assistant/message 含 args 数组。

- [x] 6.5 每回合落一条 `system/message`（完整系统提示词）；`derive_message_skeleton` 用其计 system、不再用 `context/snapshot`；账本「消息」组与 Markdown 可查看全文。验证：`one_tool_call_then_text` 断言 `system/message` 非空；`vue-tsc` 通过。

- [x] 6.6 账本标签去歧义：`message`→`model`（模型）、`request/header`→`agent`（Agent）、`system/message`→`prompt`（提示词）；禁止「助手」「系统」kind 标签。验证：`vue-tsc` 通过。

- [x] 6.7 `request/header.delta` 记录本步追加 messages；Agent 检查器/Markdown 展示；本回合末条模型文本标「最终回复」。验证：`one_tool_call_then_text` 断言两步 delta；`vue-tsc` 通过。

- [x] 6.8 Runtime 压缩策略瀑布：`keep_recent` → `summarize_tools` → `reset`；`compact/checkpoint` + `delta.compressed` 标明非追加；清理半成品重复 API。验证：compress 单测 + `vue-tsc`。

- [x] 6.9 对齐 DSH 轨迹主时间线：标签为系统/用户/上下文/助手/工具；主时间线不展示独立 Agent 行；助手/工具内联原文展开，禁止「仅工具调用/最终回复·」改写摘要。验证：`vue-tsc` 通过。

- [x] 6.10 初始系统提示词：`system/message` 带 title+tools 目录；账本摘要「初始系统提示词」；检查器「系统提示词 / 工具」页签，工具可展开 schema。验证：`one_tool_call_then_text` + `vue-tsc`。

- [x] 6.11 系统提示词/工具变更：跨回合对比上一份快照，追加「系统提示词已更新」/「工具已更新」；检查器含差异页签。验证：session_log 单测 + `vue-tsc`。

- [x] 6.12 模型调用点：有关联 `request/header` 的助手行左侧显示调用点；点击打开「请求 #N 第 M 轮」检查器（概述/选项/用量/计时）；用量未采集时显示「不可用」。验证：`vue-tsc` 通过。

- [x] 6.13 轮次术语对齐：一轮 = 用户输入→Agent 处理→交付用户；一步 = 轮内一次模型请求；请求 #N = Run 内第 N 次模型调用。UI/导出「轮」计数与标题用语一致。验证：`vue-tsc` 通过。

- [x] 6.14 主时间线紧凑单行：不内联展开；点击行在右侧检查器展示完整详情。验证：`vue-tsc` 通过。

- [x] 6.15 初始系统提示词与工具目录置于时间线最上方、不在轮次分组内；后续「已更新」仍可出现在对应轮次。验证：`vue-tsc` 通过。

- [x] 6.16 工具目录与 Skill 为独立模块：`tools/catalog` / skills `context/snapshot` 分条落库与展示；系统提示词检查器不再把工具写死为附属页签。验证：session_log + loop 单测 + `vue-tsc`。

- [x] 6.17 Skill 渐进式披露（方案 C）：对齐 Agent Skills「目录常驻 + 正文按需」；system 不含 Skill 全文；L0 `skills/catalog`、L1 `context/snapshot(skills)`；无可用通用 Rust 运行时包则 harness 自研。验证：prompt 单测 + loop + `vue-tsc`。

- [x] 6.18 系统提示词族同组分条：静态系统提示词 + 工具 + Skill 同属「系统提示词」组、标签均为「系统」、检查器分开展示。验证：`vue-tsc` + session_log 单测。

- [x] 6.19 聊天窗口过程轨默认收起为「N 次工具调用 · M 条消息」摘要，点击展开；流式中默认展开；正文始终可见。验证：`vue-tsc`。

- [x] 6.20 Skill 检索意图聚焦：超长粘贴抽出全文自然语言意图句（前/后皆可）打分，避免 DSL 泛词抢位；`json.format` 补「转义/反转义」路由；工具宽松解析已转义输入；导出按 `systemChange` 区分摘要。验证：skills + json 单测 + `vue-tsc`。

- [x] 6.21 顶栏多轨时间线：`buildSwimlaneBlocks` 投影输入/模型/工具三轨；`AgentRunTrajectory` 工具栏下方渲染泳道，点击选中检查器；搜索联动 dim/hit。验证：`vue-tsc` 通过。
