## Context

TBox 已经在 `agent-chat` capability 下完整持久化了 Agent 轨迹：每条助手消息的 `trajectory_json` / `tool_calls_json` / `reasoning` / `content` 都可由 `get_conversation_messages` 拉回（`src-tauri/src/commands/conversation.rs:401`），并通过 `AssistantTrajectory` 组件在聊天首页按时间线渲染（`src/components/AssistantTrajectory.vue`）。本 change 在持久化层引入三层交互节点模型，并在前端改造组件渲染与导出。

**数据模型绑定**（UI 与命名据此设计）：

| 层级 | 概念 | 实体 | 关系 |
|---|---|---|---|
| 外层 | Run | Conversation（`conversations` 表一行） | 1 个 Run = 1 个 Conversation |
| 中层 | Trajectory | Assistant message | 1 个 Run 含 N 个 Trajectory（N = 助手回合数） |
| 中层（per trajectory） | ModelCall | `trajectory_json` 的一项 `{type:"model_call"}` | 1 个 Trajectory 含 M 个 ModelCall（M = 该回合模型调用次数） |
| 内层（per model call） | ToolStep | ModelCall.tools[] 一项 | 1 个 ModelCall 含 K 个 ToolStep（K = 该次模型调用 dispatch 的工具数） |

**三层交互映射**：
- `USER → AGENT` ↔ 每条 user message（外层会话回合）
- `AGENT → MODEL` ↔ 每个 ModelCall 节点（loop.rs 一次 `complete_streaming` 调用 = 一次迭代）
- `AGENT → TOOL` ↔ 每个 ToolStep 节点（注册表内某工具的一次 dispatch）

前端列表用 `loadList()` 拿到 Run 元信息，详情用 `openConversation(id)` 拿到该 Run 的全部消息（含其全部 ModelCall 集合）。任何路径下都不会出现「单条 Trajectory / ModelCall 作为顶层入口」或「Run 与 Trajectory 解耦展示」。

数据来源完全在前端可达：下载沿用 `@/utils/download` 的 `downloadTextFile`（19 行），剪贴板沿用既有 `useClipboard.ts`。

**Rust 侧持久化改造要点**（向后兼容）：
- 新增 `TrajectoryNode::ModelCall { id, reasoning?, tools: Vec<ToolStep>, response?, stream_mode? }` + `ToolStep { id, args, result?, status }`（`agent/trajectory.rs`）
- 新增 `build_trajectory_nodes_from_steps(steps, boundaries, stream_mode)` 把 flat `Vec<TrajectoryStep>` + 迭代边界转成 `Vec<TrajectoryNode>`
- `resolve_trajectory` 返回类型升级为 `Vec<TrajectoryNode>`，优先解析新格式，旧格式 `Vec<TrajectoryStep>` 自动降级为单个合成 ModelCall
- `loop.rs` 维护 `iter_boundaries: Vec<usize>`，每轮迭代开始时 `push(trajectory.len())`；持久化时调用 `build_trajectory_nodes_from_steps` 生成新 JSON
- 旧数据：旧库消息无 `trajectory_json` 时由 `reasoning`/`tool_calls_json`/`content` 合成降级时间线（旧 spec 行为）；旧 `trajectory_json` 为 legacy flat 数组时包装成单个 ModelCall

## Goals / Non-Goals

**Goals**
- 新增 `/agent-runs` 列表页 + `/agent-runs/:id` 详情页，复用既有 trajectory / 消息组件，不重复实现时间线渲染逻辑。
- 聊天首页（`HomePage`）当前会话视图提供「查看完整轨迹」主入口，跳转到 `/agent-runs/:id`；侧边栏的「Agent 运行」作为跨会话浏览的次入口保留。
- 一键导出 Markdown / JSON + 复制为 Markdown，全部纯前端装配，零 Rust 变更、零新依赖。
- **详情页对齐 DeepSeek Harness `ui-trajectory` 账本**：Turn → 消息/第 M 步分组，紧凑 `#N` 记录行（用户/助手/工具/上下文/已压缩）+ 右侧检查器；**顶栏 SHALL 展示「输入 / 模型 / 工具」三轨时间线**，并与搜索联动高亮/弱化。
- 删除会话通过既有 FK 级联自动清理轨迹，spec 显式声明该契约。
- 不改聊天首页的输入与发送逻辑、不改压缩、不动工具注册。

**Non-Goals**
- 不做实时回放、不做时间线过滤、不做多会话对比。
- 不动 SQLite schema、不动 Rust command、不动 Tauri 权限。
- 不引入 PDF / zip / 二进制导出（v1 仅 Markdown 与 JSON）。
- 不在 `/toolbox` 网格注册为普通工具。
- 不引入"model 调用边界"作为顶层节点（当前数据模型无法区分同一回合内的多次模型调用；保留 flat 节点 + 节点内字段）。
- **不对每步 dump 完整 `messages` 快照**；系统提示词作 surface history；发给模型的文案在 `request/header.delta` 记追加，或压缩后记 `compressed: true` + 策略。Runtime 压缩瀑布：`keep_recent` → `summarize_tools` → `reset`；system 恒保留；Audit 不受影响。

## Decisions

- **路由合并到一个 Vue 文件**：列表与详情合并到 `src/views/AgentRuns.vue`，内部用 `route.params.id` 是否存在切换两种视图。理由：v1 仅两个状态、无独立 SEO / 独立缓存需求，分文件会增加 router 配置与导航心智而无收益。
- **主入口放在聊天首页的会话内**：排查入口紧贴「我刚刚聊到哪里」的语境，比侧边栏全局入口更自然；用户点击消息后立刻能看到「这段对话出了什么」的入口。侧边栏全局入口保留为「跨会话浏览所有 Run」的次入口，避免把用户卡死在某个会话里找历史。
- **侧边栏新入口而非放到 toolbox**：与第一版决策一致；排查入口与正式工具语义不同。详情页顶部「返回列表」「返回聊天」两个入口负责回到主流程。
- **详情页默认折叠（与第一版差异）**：第一版默认全展开，定位「排查视图一眼可见」；用户反馈希望对齐 deepseek-harness / 聊天首页的折叠策略——reasoning/tool 节点默认收起、text 节点（最终正文）保持可见。AssistantTrajectory 组件 `streaming=false` 时已默认此行为，**无需在 AgentRuns 写自定义覆盖**；只需把 spec 从「默认全部展开」改回「默认全部折叠」。用户手动展开状态不跨会话持久化（组件 watch 行为已保证）。
- **每个 trajectory 节点对齐 deepseek-harness 风格**：节点 = reasoning（agent↔model 推理）/ tool（agent↔tool 调度）/ text（agent↔model 最终答复），与 deepseek-harness 风格一致：每个节点独立可折叠、正文始终可见。Markdown 导出同样按节点组织，便于贴 issue 后对方点开看。
- **Markdown vs JSON 二选一而非合并为单文件**：Markdown 给人类看、JSON 给程序/MCP 解析，格式差异大合并无收益。提供两个按钮 + 一个复制 Markdown 按钮（共 3 个导出动作），不引入二级菜单。
- **Markdown 中保留推理为引用块、工具块用子段 + 代码块**：与 GitHub/GitLab 渲染兼容，便于贴 issue；JSON 中保留 `trajectory_json` 原始结构（`get_conversation_messages` 已返回），不二次归一化，便于对照 `agent-trajectory.rs` 的 `TrajectoryStep` 枚举。
- **不写新的 Rust command**：理由是数据已经在 `get_conversation_messages` 返回体里，再写一个 `export_conversation_markdown` 命令并不会减少前端代码量，反而引入 IPC 序列化 + 模板字符串双端维护成本。前端组装是更直接的选择。
- **删除级联走既有 FK，不写新 Rust**：既有 `FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE`（`src-tauri/src/commands/conversation.rs:47`）已保证删除 Run 时级联清空 messages（含 trajectory_json）。仅在 spec 显式声明该契约，无需新增 command 或 schema 变更。

## Risks / Trade-offs

- [Risk] Markdown / JSON 在前端组装，**模板与 Rust 端 `TrajectoryStep` 枚举耦合**（字段名变更需要同步改导出逻辑） → Mitigation：把组装函数（`buildAgentRunMarkdown`、`buildAgentRunJson`）放进 `src/utils/agentRunExport.ts`，命名刻意与 Rust `agent/trajectory.rs` 对齐；在该文件顶部加注释提醒「字段调整时同步 Rust 端」。并在 tasks 阶段给该工具函数加最少一两条 vitest（可选）。
- [Risk] 大会话导出 Markdown 可能触发浏览器内存峰值（每条工具结果按字面塞进代码块） → Mitigation：v1 接受该限制（典型本地工具调用单条结果 << 10KB，会话数 << 数千条），后续若需要可在 utils 里对单条 `result` 截断 + 标注「[truncated]」。
- [Risk] 用户把含 token / API key 的会话误导出并外传 → Mitigation：导出按钮文案明确「本地导出」图样；spec 增加「No cloud upload on export」Scenario；JSON 不带任何环境变量；trajectory 本身不含密钥（凭据只存在设置页 profile 内，不进消息字段）。
- [Trade-off] 详情页默认折叠后，用户排查时需逐个点开每个节点才能看到推理/工具内容 → Mitigation：`AssistantTrajectory` 已有点击整条（chevron）展开，单击即开；典型 Run 节点数 << 几十，排查成本可接受；后续若需要可加「一键展开全部」按钮。
- [Trade-off] 主入口放在聊天首页而非侧边栏，会让「跨会话浏览 Run」的视觉权重降低 → Mitigation：侧边栏全局入口仍可见（`/agent-runs` 入口），仅作为次入口；不删除以保留跨会话浏览能力。

## Migration Plan

纯前端增量、无 schema 变更、无 Rust 变更。回滚：删除新增文件 + router / SideBar 的两处小改动即可，不影响其他功能。

## Open Questions

- 是否需要在 v1 之外再支持「跨会话批量导出」？（先不做，避免一次性带太多命令）
- 是否要顺便在 `agent-run-trajectory-export` 同 change 内补一条对 `Trajectory Persistence` 的 sub-scenario，明确要求 `messages.trajectory_json` 对历史压缩后的工具结果仍保存原始 Result？（目前 spec 已隐含：spec line 246「compressed items remain accessible」已要求，但没明确 export 维度。可在 archive 阶段顺手补一行 Scenario 在此 change 的 spec 里。）
