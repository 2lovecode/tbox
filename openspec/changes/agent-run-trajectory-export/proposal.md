## Why

Agent 轨迹（reasoning + tool calls + 正文）目前只能在聊天首页实时流式看到；历史会话重开后轨迹虽然保留（见 `agent-chat` 的 `Trajectory Persistence` 与 `Assistant Turn Trajectory Display`），但用户遇到 agent 跑歪、工具报错、循环卡死等问题时，缺乏一个跨会话、集中查看并「把整段轨迹带出去」排查的入口。补一个独立的 Agent 运行轨迹展示页与一键导出，可以让用户在不去翻 SQLite 的前提下复盘每一步模型决策、工具参数与结果。

## What Changes

**数据模型绑定（务必先看）**：在本 change 语义里，**一次 Run = 一个会话（Conversation）**，`runs` 与 `conversations` 一一对应；**一个 Run 包含一组 Trajectory**，每个 Trajectory 对应该 Run 内的一次助手回合（一条 assistant message 的 `trajectory_json`）。轨迹与 Run 不可解耦：列表的每一行是一个 Run，详情页展示该 Run 的完整 trajectory 集合（按消息时间顺序排列的所有助手回合）。

**三层交互模型（关键修订）**：当前实现把「agent ↔ model」的 CoT/正文和「agent ↔ tool」的调度都混在 `trajectory_json` 的 flat 数组里，丢失了「一次交互」的边界。本版改造为**扁平事件列表**，每种交互都是一条独立节点，一条条排列，无嵌套：

| 交互 | 节点类型 | 一条节点代表什么 |
|---|---|---|
| 用户 ↔ agent | 消息行（UI 标签「USER → AGENT」） | 一条 user message（从消息列表渲染，不写入 trajectory_json） |
| agent ↔ model | `model_call` 事件 | 一次 `complete_streaming` 调用（loop.rs 中的一次迭代）：reasoning、requested_tools、response、stream_mode |
| agent ↔ tool | `tool_call` 事件 | 注册表内某个工具的一次 dispatch：参数 → 结果或失败 |

落库形态（`messages.trajectory_json`）：
- **新格式（扁平事件数组）**：`[{type:"model_call",...},{type:"tool_call",...},{type:"model_call",...}]` —— model_call 与其请求的 tool_call 事件依次排列（model_call 在前、其 tool_call 紧随其后），每次交互一条
- **旧格式兼容**：legacy flat `[{type:"reasoning"|"tool"|"text"}]` → 启发式转换（每个 reasoning 开启新 model_call、tools 挂在其后、末尾 text 为最终回复）；开发期短暂存在的嵌套格式（model_call 内嵌 tools 数组）→ 读侧拍平

**入口形态**：v1 计划在侧边栏底部放一个全局「Agent 运行」入口；本版调整为主入口**放进聊天首页（HomePage）当前会话内**——只要 `conversations.activeId` 非空、且至少有一条消息，聊天界面 SHALL 提供一个「查看完整轨迹」入口跳到 `/agent-runs/:id`。侧边栏的全局入口保留作为「跨会话浏览所有 Run」的非主入口，可后续讨论是否移除。

**删除级联**：删除会话时，对应的所有消息（含 trajectory_json / reasoning / tool_calls_json / content）通过既有 `FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE`（`src-tauri/src/commands/conversation.rs:47`）自动级联删除，**无需新增 Rust 逻辑**。本 change 仅在 spec 中显式声明该行为作为契约。

**默认折叠（与第一版差异）**：第一版 spec 写「详情页默认全部展开」以「凸显排查定位」；本版调整为**默认全部折叠**，与聊天首页 `Assistant Turn Trajectory Display` 的「完成后默认折叠」策略一致——每个 ModelCall 节点默认收起，用户点开某个节点才展开；正文（response）作为最终答复保留可见，避免整段不可读。整体观感对齐 deepseek-harness 风格的折叠轨迹树。

具体改动：

- 新增路由 `/agent-runs`：进入后默认展示当前所有 Run（即全部会话，按 `updated_at DESC`）的扁平列表，每行展示标题、时间、消息数、轨迹数（=助手回合数）、是否含工具调用、是否被压缩过等元信息；点击行进入该 Run 的轨迹详情页。
- 新增路由 `/agent-runs/:id`：以时间线形式逐条渲染该 Run 的所有消息（用户消息 + 助手回合），每个助手回合复用既有的 `AssistantTrajectory` 组件渲染 `ModelCall[]`，**每个 ModelCall 节点默认收起**（沿用组件默认 `streaming=false` 行为）；工具块完整展示参数与结果；右上角放「导出」按钮，导出范围是「该 Run 的全部轨迹集合」，不是单个轨迹。
- 新增一键导出：单 Run 支持导出 Markdown（默认）与 JSON 两种格式，文件名 `<title>-<id短截>-<日期>.md|.json`；复用既有 `downloadTextFile()` 通过 WebView 触发下载，不引入新的 Tauri 权限或对话框插件。
- 新增「复制为 Markdown」：复制按钮把同一份 Markdown 写入剪贴板，便于直接粘贴到 issue。
- 新增「聊天首页会话内入口」：在 `HomePage.vue` 当前会话元信息区（消息列表上方）新增「查看完整轨迹」按钮/链接，跳转 `/agent-runs/:id`；仅当 `conversations.activeId` 非空且至少有一条消息时显示。
- 删除会话通过既有 `delete_conversation` command + FK 级联自动级联删除对应 message 行（含 trajectory_json / reasoning / tool_calls_json / content）。本 change 不改 Rust 侧，仅在 spec 显式声明该契约。
- 不引入 Boxes/Toolbox 工具注册（不放在 `/toolbox` 网格里）：这是 agent 排查专用入口。

**Rust 侧最小侵入改造**：
- `agent/trajectory.rs`：新增 `TrajectoryNode::ModelCall` + `ToolStep`；新增 `build_trajectory_nodes_from_steps(steps, boundaries, stream_mode)` 把 flat `Vec<TrajectoryStep>` + 迭代边界转成 `Vec<TrajectoryNode>`；`resolve_trajectory` 返回类型升级为 `Vec<TrajectoryNode>`，优先解析新格式，旧格式按"包成单个 ModelCall"降级
- `agent/loop.rs`：维护 `iter_boundaries: Vec<usize>`，每轮迭代开始时 `push(trajectory.len())`；持久化时调用 `build_trajectory_nodes_from_steps` 生成新 JSON；`stream_mode` 在每次迭代记录并 attach 到对应 ModelCall
- 不改 SQLite schema、不改现有 command、不改 agent event wire format（前端看到的还是 `AgentEventPayload`，由前端基于 `trajectory_json` 重新渲染）

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `agent-chat`：新增 2 个 Requirement ——「Agent Run Trajectory Page」覆盖列表页与详情页的导航/展示行为；「Agent Run Trajectory Export」覆盖 Markdown/JSON 导出与复制行为。两个 Requirement 不改既有 trajectory 持久化或聊天首页行为，纯粹叠加在已有 `Trajectory Persistence` / `Assistant Turn Trajectory Display` 之上。

## Impact

- 前端：
  - 新增 `src/views/AgentRuns.vue`（列表 + 详情二级路由合并页）或拆成 `AgentRunsList.vue` + `AgentRunDetail.vue`，倾向合并到一个 Vue 文件用内部状态切换，更简单。
  - `src/router/main.ts`：新增 `/agent-runs` 与 `/agent-runs/:id`（动态 import）。
  - `src/layout/SideBar.vue`：在历史列表底部增加「Agent 运行」入口，路由跳转 `/agent-runs`。
  - 新增 `src/utils/agentRunExport.ts`：从 `ChatMessage[]` 组装 Markdown / JSON 字符串。
  - 复用：`@/utils/download` 的 `downloadTextFile`、`@/components/AssistantTrajectory`、`@/utils/trajectory` 的 `resolveTrajectory`、`@/stores/conversations` 的 `loadList` + `openConversation`。
- 后端：无新增 / 修改 Rust 命令，纯前端装配。
- SQLite schema：无变更；导出依赖的 `messages.trajectory_json` / `messages.tool_calls_json` / `messages.reasoning` 已存在。
- 不影响：聊天首页、Spotlight、设置页、toolbox 工具网格、压缩逻辑、模型循环。

## Non-goals

- 不在 `/agent-runs` 做实时流式回放（历史视角，无可重放事件源）。
- 不引入 diff / 多会话对比 / 时间线过滤（v1 仅按会话列表 + 单会话详情）。
- 不改变聊天首页或轨迹存储结构。
- 不写新的 Tauri 命令、不加新依赖（PDF / zip / 等）。
- 不向云端上传任何轨迹，纯本地导出。
- 不在 `/toolbox` 网格注册为「工具」（这是排查页，不是普通工具）。
