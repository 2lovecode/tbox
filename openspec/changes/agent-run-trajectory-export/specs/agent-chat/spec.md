## ADDED Requirements

### Requirement: Session Event Log

Agent 循环 SHALL 以**仅追加事件日志（append-only event log）**作为轨迹的事实源（参考 DeepSeek Harness 的「日志即事实源」设计）：每次交互（用户输入、模型请求/回复、工具调用/结果、上下文注入、压缩）都 MUST 以一条 `SessionEvent` 追加写入本地 SQLite `session_events` 表，历史事件 MUST NOT 被修改或删除；删除会话时其事件经 FK 级联一并删除。

**事件结构**：每条事件 MUST 含 `seq`（会话内全局递增序列号）、`time`（Unix 毫秒时间戳）、`type`（事件类型）、`data`（事件载荷 JSON）。事件类型体系：

| type | 时机 | data 关键字段 |
|---|---|---|
| `turn/start` | 一轮任务开始 | turn_no |
| `step/start` | 一次模型请求开始 | turn_no, step_no |
| `user/message` | 用户输入 | content |
| `system/message` | 本回合最终系统提示词（含 memory 注入；**不含**工具目录模块） | content |
| `tools/catalog` | 工具目录模块（与系统提示词分离） | tools, previous_tools?, title, change |
| `request/header` | 模型请求头快照 | model, backend, message_count, tools, stream_mode, desync_check, **delta?** |
| `assistant/message` | 模型完整回复（文本或工具调用决策） | reasoning?, text?, tool_calls? |
| `tool/call` | 工具调用 | tool_id, args |
| `tool/result` | 工具返回 | tool_id, ok, result |
| `context/snapshot` | 上下文模块（memory / **skills L1** 等） | source, chars, skills? |
| `skills/catalog` | Skill L0 目录（name+description，与正文分离） | skills: [{id,name,description}], title, change |
| `compact/checkpoint` | Runtime 上下文压缩 | count, message, kind: `"compress"`, strategies[] |
| `step/end` | 一次模型请求结束 | turn_no, step_no |
| `turn/end` | 一轮任务结束 | turn_no |

**请求增量 `delta`（忠实因果链）**：每次发起模型请求时，`request/header.data.delta` SHALL 含 `messages` 数组。

- 无压缩：第 1 步仅为本回合 user；第 2 步+ 为相对上一次模型请求后追加的切片（不含 system 重放）
- **本步发生压缩**：MUST 设 `compressed: true` 与 `strategy`（瀑布中最后生效的策略）；`messages` 为压缩后 Runtime 中 system 之后的视图；若策略为 `reset` 另设 `reset: true`。账本 Agent 行 MUST 标「压缩后请求」，MUST NOT 标普通「追加」

账本目标：读完一条 Run 须能还原「用户原文 → Agent 处理 → 模型调用（追加或压缩后视图 + 输出）→ 工具 I/O → 最终给用户的回复」。

**日志重建一致性校验（log-reconstruction desync check）**：Agent 循环在每次发起模型请求前，SHALL 从事件日志推导模型上下文骨架（system/user/assistant/tool 消息的角色与条数），与实际待发送请求比对；校验结果 MUST 记录在该次 `request/header` 事件的 `desync_check` 字段（`{ derived_messages, actual_messages, match: bool }`）。校验不一致时 MUST NOT 静默通过——以警告日志 + `request/header` 标记 `match:false` 呈现（不阻断请求）。

**兼容**：既有 `messages.trajectory_json` 扁平事件格式继续由 Agent 循环写入与回放；`/agent-runs` 详情页 SHALL 优先读取 `session_events` 日志渲染，无日志数据时 MUST 回退到 `trajectory_json` 推导（旧数据兼容）。导出 JSON SHALL 附带 `sessionEvents` 原始数组。

#### Scenario: Events appended per interaction in order
- **WHEN** 用户发送一条需要一次工具调用的消息并得到最终回复
- **THEN** `session_events` 表中该会话按 seq 递增依次出现 turn/start → user/message → system/message → step/start → request/header → assistant/message(tool_calls) → tool/call → tool/result → step/end → step/start → request/header → assistant/message(text) → step/end → turn/end，seq 严格递增且无空洞

#### Scenario: System prompt recorded once per turn
- **WHEN** Agent 循环完成本回合系统提示词拼装（含可选 memory 注入）并即将发起首次模型请求
- **THEN** 追加恰一条 `system/message`，其 `data.content` 为发给模型的完整 system 文本；回合内后续 step MUST NOT 再追加 `system/message`；`context/snapshot` 仅作 memory 元数据，MUST NOT 再计入 desync 的 system 角色

#### Scenario: Request header carries incremental delta
- **WHEN** 用户发送一条需一次工具调用再总结的消息
- **THEN** 第 1 步 `request/header.delta.messages` 仅含本回合 user 一条；第 2 步 `delta.messages` 含 assistant(tool_calls) 与对应 tool 结果，不含 system、不含第 1 步已展示的 user 原文重复

#### Scenario: Append-only guarantee
- **WHEN** 任何后续操作（压缩、重开、继续对话）发生
- **THEN** 已有事件的 seq/type/data MUST 保持不变；新信息只以更高 seq 追加

#### Scenario: Desync check recorded per request
- **WHEN** Agent 循环发起任一次模型请求
- **THEN** 该次 `request/header` 事件的 `desync_check` 含从日志推导的消息骨架计数与实际请求计数，二者一致时 `match:true`；不一致时 `match:false` 且不阻断该请求

#### Scenario: Compact checkpoint recorded without mutating history
- **WHEN** 上下文接近阈值触发 Runtime 压缩瀑布
- **THEN** 一条 `compact/checkpoint`（kind=compress，含 strategies）追加；此前所有事件（含 tool/result 原文）保持原样可查；随后 `request/header.delta.compressed` 为 true

#### Scenario: Cascade delete removes events
- **WHEN** 用户删除某个会话
- **THEN** 该会话的 `session_events` 行经 FK 级联全部删除，不残留

#### Scenario: Detail page prefers event log with fallback
- **WHEN** 用户打开 `/agent-runs/:id`
- **THEN** 详情页优先以 `session_events` 日志渲染单一事件流；该会话无事件日志（旧数据）时回退到 `trajectory_json` 扁平事件推导，界面不崩溃

### Requirement: Agent Run Trajectory Page

**数据模型绑定**：在本能力语义中，**一次 Run = 一个会话（Conversation）**，`runs` 与 `conversations` 一一对应；**一个 Run 包含一组 Trajectory**，每个 Trajectory 对应该 Run 内的一次助手回合（一条 assistant message 的 `trajectory_json`）。Run 与 Trajectory MUST NOT 解耦展示——列表的每一行是一个 Run，详情页呈现的是「该 Run 的完整轨迹集合」，不会把单条 Trajectory 单独作为顶层入口。

**三层交互节点模型（关键，扁平无嵌套、单一时间线）**：本能力定义的轨迹 MUST 为**扁平事件列表**——每种交互都是一条独立节点，一条条依次排列，MUST NOT 嵌套，也 MUST NOT 在用户消息与助手回合外再包「USER → AGENT / AGENT → USER」两大类分组：整个 Run 是一条从上到下的单一事件流：

| 交互 | 节点 | 一条节点代表 |
|---|---|---|
| 用户 ↔ agent | `user_message` 行 | 一条 user message（从消息列表顺序内联渲染，不写入 trajectory_json） |
| agent ↔ model | `model_call` 事件 | 一次 `complete_streaming` 调用（loop.rs 中的一次迭代），含 reasoning、requested_tools、response、stream_mode |
| agent ↔ tool | `tool_call` 事件 | 注册表内某个工具的一次 dispatch（参数 → 结果或失败） |

落库形态（`messages.trajectory_json`）：
- **新格式（扁平事件数组）**：`[{type:"model_call",...},{type:"tool_call",...},...]` —— model_call 与其请求的 tool_call 依次排列（model_call 在前、其 tool_call 紧随其后），每次交互一条
- **旧格式兼容**：legacy flat `[{type:"reasoning"|"tool"|"text"}]` → 启发式转换（每个 reasoning 开启新 model_call、tools 挂其后、末尾 text 为最终回复）；开发期嵌套格式（model_call 内嵌 tools 数组）→ 读侧拍平

**入口契约**：系统 SHALL 仅通过会话关联的详情页触达轨迹（无跨会话列表页）：
1. **聊天首页会话内入口（主入口）**：在 `HomePage` 提供「轨迹」入口，当 `conversations.activeId` 非空且消息数 ≥ 1 时可见，跳转 `/agent-runs/:id`。
2. **侧栏右键「打开轨迹」**：跳转同一详情页 `/agent-runs/:id`。
3. **无列表**：MUST NOT 提供 `/agent-runs` 列表页作为产品入口；访问无 id 的 `/agent-runs` MUST 重定向到聊天首页。详情页返回 SHALL 仅回到该会话聊天（`/` + 打开对应 conversation），MUST NOT 提供「返回列表」。

**详情页（对齐 DeepSeek Harness `ui-trajectory` 扁平事件流）**：详情页 SHALL 把 session_events（无日志时回退 trajectory_json）投影为 **按时间顺序的扁平事件流**（可按「第 N 轮」弱分组，视觉上是连续日志，不是 AGENT→MODEL 大卡片）：

**轮次 / 步 / 请求（术语）**：

| 概念 | 定义 | 事件边界 | UI |
|---|---|---|---|
| **轮次（Turn）** | 用户给出一次输入，Agent 经一系列处理（可含多次模型调用与工具）后把结果交付给用户 | `turn/start` … `turn/end`；`turn_no` 在会话内递增 | 「第 N 轮」弱分组；MUST NOT 把单次模型调用当成一轮 |
| **步（Step）** | 一轮内的一次模型请求 | `step/start` … `step/end`；`step_no` 在该轮内从 1 递增 | 导出可用「第 M 步」；主时间线可扁平不显式标步 |
| **请求 #N** | Run 内第 N 次模型调用（跨轮连续编号） | 与某次 `request/header` + 随后 `assistant/message` 对应 | 调用点检查器标题「请求 #N · 第 M 轮」 |

一轮可含多步；一步对应一个调用点。列表与详情中的「轮」计数 SHALL 优先取 session 投影中的非空 `turn` 个数；无事件日志时可用「用户消息数」或「助手消息数」近似（通常一一对应）。

| kind | 标签（DSH） | 来源 |
|---|---|---|
| `user` | 用户 | `user/message` |
| `prompt` | 系统 | `system/message`（系统提示词全文） |
| `context` | 上下文 | `context/snapshot` |
| `model` | 助手 | `assistant/message`（思考 / 正文 / tool_calls，**原文如实**） |
| `tool` | 工具 | `tool/call` + `tool/result` 配对 |
| `compacted` | 已压缩 | `compact/checkpoint` |

`request/header`（含 delta）SHALL 保留在事件日志中供校验/导出，主时间线 MUST NOT 以独立「Agent」行展示。

**模型调用点（对齐 DSH）**：每次成功关联到 `request/header` 的助手记录 MUST 在时间线左侧 gutter 显示可点击的**调用点**（小方点/圆点）。点击调用点 MUST 打开右侧检查器，标题形如「请求 #N · 第 M 轮」（N = Run 内模型调用序号、M = **所属轮次**，即用户输入→交付输出的那一轮），页签为 **概述 / 选项 / 用量 / 计时**：

| 页签 | 内容 |
|---|---|
| 概述 | 状态、提供方（backend）、模型、本步工具调用数、结果（助手消息 / 工具调用） |
| 选项 | `provider`/`model` 等请求选项 JSON（来自 `request/header`） |
| 用量 | 输入/缓存/输出 token；当前运行时未采集时 MUST 显示「不可用」，MUST NOT 伪造数字 |
| 计时 | 开始时间、总时长（由 `step/start`/`request/header` 与 `assistant/message` 时间差得出）；首 token / 吞吐未知时显示「不可用」 |

点击调用点与点击行正文选择态可并存但检查器模式不同：点 → 调用元数据；行 → 既有内容检查器。无关联 `request/header` 的旧回退数据可不显示调用点。

标签约定：主时间线使用「系统 / 用户 / 上下文 / 助手 / 工具」（对齐 DeepSeek Harness）；内部 kind 仍为 `model`。

每条记录为**紧凑单行**：左侧 kind 色标 + 单行摘要（ellipsis）+ 耗时；**主时间线 MUST NOT 内联展开**思考/正文/工具参数与结果。点击行 MUST 在右侧检查器打开完整详情（模型：思考/输出/tool_calls；工具：参数/结果；系统/用户：原文）。模型摘要 MUST 使用事件中的原始 `reasoning` / `text` / `tool_calls` 的预览，MUST NOT 改写成「仅工具调用 ·…」「最终回复 ·…」；若需标示本回合最后一条模型文本，可用角标「交付用户」，不得改写正文。旧日志若 `assistant/message` 未带思考，SHALL 用同会话 `trajectory_json` 的对应 `model_call.reasoning` 回填。工具栏 SHALL 提供：时长摘要、收起/展开全部轮次、收起/展开工具调用、**搜索**。重进 Run 时选择态与折叠态 MUST 复位。

**顶栏多轨时间线（SHALL）**：详情页工具栏下方 SHALL 展示「输入 / 模型 / 工具」三轨水平时间线（泳道）：
- **输入**：用户消息、系统提示词、上下文、压缩、Agent 请求头等非模型/非工具事件
- **模型**：助手 / 模型调用块
- **工具**：工具调用块
色块按事件时间（或缺时间戳时按顺序）映射到相对 Run 轴；点击色块 MUST 选中对应主时间线记录并打开检查器。搜索生效时，不匹配色块 MUST 弱化（dim），匹配色块 MUST 高亮。

#### Scenario: System prompt visible in ledger and export
- **WHEN** 用户打开含 `system/message`（initial）事件的 `/agent-runs/:id` 或导出 Markdown
- **THEN** 时间线**最上方**（轮次分组之外）出现「系统提示词」组，组内分条展示「静态系统提示词」；点开后检查器为「静态提示词」页签；MUST NOT 把初始静态提示词挂在「第 N 轮」内；无该事件的旧 Run MUST 不崩溃、不伪造

#### Scenario: Tools catalog is a separate module
- **WHEN** Agent 首次落库工具目录或工具目录相对上次变更
- **THEN** 追加 `tools/catalog`（标题「工具已加载」或「工具已更新」），与静态系统提示词同属「系统提示词」组、分条展示；检查器以「工具」页签展示 schema；MUST NOT 将工具目录写死进 `system/message` 事件正文

#### Scenario: Skills load as system-prompt family rows
- **WHEN** 本回合按用户输入检索到 Skill，或落库 Skill L0 目录
- **THEN** L0 经 `skills/catalog`、L1 经 `context/snapshot(source=skills)` 落库；主时间线标签为「系统」，与静态提示词/工具同属「系统提示词」组、分条展示；检查器可查看技能目录或正文；MUST NOT 把 Skill 仅当作无关「上下文」行

#### Scenario: Skills progressive disclosure (catalog + on-demand body)
- **WHEN** Agent 构建本轮发给模型的上下文
- **THEN** 启用 Skill 的 **L0 目录**（name + description）经 `skills/catalog` 落库并可在会话顶「系统提示词」组展示；**L1 正文**仅在 `retrieve_skills` 命中后注入模型上下文并经 `context/snapshot(source=skills)` 落库；`system/message` 正文 MUST NOT 含「相关技能说明」全文块；无命中时 MUST NOT 注入 L1

#### Scenario: System prompt or tools change emits system row
- **WHEN** 同一会话后续回合的系统提示词正文或工具目录相对上一份快照发生变化
- **THEN** 分别追加 `system/message`（「静态系统提示词已更新」）和/或 `tools/catalog`（「工具已更新」）；未变化时 MUST NOT 重复追加静态系统提示词或工具已加载

#### Scenario: Ledger labels match DSH roles
- **WHEN** 用户打开含模型调用与工具调用的 Run 详情
- **THEN** 主时间线 kind 标签为「用户 / 系统 / 上下文 / 助手 / 工具」（及可选「已压缩」）；MUST NOT 出现独立「Agent」主行；LLM 输出行标签为「助手」而非「模型」

#### Scenario: Model output shown faithfully
- **WHEN** 用户查看含 `assistant/message` 的助手记录
- **THEN** 主时间线仅显示原文预览摘要；点开后检查器展示事件中的原始思考/正文/tool_calls；摘要 MUST NOT 被改写成「仅工具调用」或「最终回复 ·」前缀句

#### Scenario: Compact rows open details on the right
- **WHEN** 用户打开 Run 详情且未选中任何记录
- **THEN** 主时间线每条记录为单行摘要，无内联展开正文；点击某行后右侧出现该记录的完整详情检查器

#### Scenario: Request header stays in event log
- **WHEN** 导出 JSON 或查看 `sessionEvents`
- **THEN** `request/header.delta` 仍存在于原始事件中；主时间线不依赖独立 Agent 行即可读懂模型与工具因果

#### Scenario: Final model reply marked
- **WHEN** 一轮含工具调用并以模型文本结束
- **THEN** 该轮最后一条带输出文本的助手记录可用角标标明交付用户；正文仍为 LLM 原文

#### Scenario: Model call marker opens request inspector
- **WHEN** 用户打开含至少一次 `request/header` + `assistant/message` 的 Run，并点击某条助手行左侧的调用点
- **THEN** 右侧打开「请求 #N · 第 M 轮」检查器，含概述/选项/用量/计时；概述展示提供方与模型；用量在未采集时为「不可用」；M 为该条记录所属用户输入轮次，MUST NOT 等于步号

#### Scenario: One user input is one turn
- **WHEN** 用户发送一条消息，Agent 在该轮内多次调用模型并使用工具后给出最终回复
- **THEN** 账本仅出现一个「第 N 轮」分组；组内可有多条助手/工具记录与多个调用点；MUST NOT 为每次模型调用新开一轮

#### Scenario: Call marker absent without request header
- **WHEN** 仅有 `trajectory_json` 回退、无 `request/header` 的旧助手记录
- **THEN** 该行 MUST NOT 显示调用点；点击行仍可在右侧检查器查看正文

**删除级联（契约性声明）**：调用既有 `delete_conversation` 删除某个会话时，对应的全部消息（含 `trajectory_json` / `reasoning` / `tool_calls_json` / `content`）以及 `session_events` MUST 通过既有 `FOREIGN KEY ... ON DELETE CASCADE` 自动级联删除；删除后 MUST NOT 再通过 `/agent-runs/:id` 访问到任何残留轨迹。

#### Scenario: Turn-aware ledger grouped by step
- **WHEN** 用户打开 `/agent-runs/:id` 且该会话含多条 user message 与至少一次工具调用
- **THEN** 页面可按「第 N 轮」弱分组，组内为扁平系统/用户/上下文/助手/工具记录；每条记录左侧 kind 色标、中间单行摘要；点击行在右侧打开详情；不使用「AGENT → MODEL / AGENT → TOOL」大卡片，主时间线无独立 Agent 行、无内联展开正文

#### Scenario: Multi-track swimlane timeline
- **WHEN** 用户打开含用户输入、模型调用与工具调用的 Run 详情
- **THEN** 工具栏下方 SHALL 显示「输入 / 模型 / 工具」三轨水平时间线；输入轨含用户/系统/上下文等块，模型轨含助手块，工具轨含工具块；点击任一色块 MUST 选中对应主时间线记录并打开检查器

#### Scenario: Trajectory search filters ledger and dims swimlane
- **WHEN** 用户在轨迹工具栏搜索框输入关键词
- **THEN** 主时间线仅保留匹配记录；多轨时间线中不匹配色块弱化、匹配色块高亮；清空搜索后全部恢复

#### Scenario: Per-conversation entry visible after first message
- **WHEN** 用户在聊天首页发送了至少一条消息（`conversations.activeId` 非空且消息数 ≥ 1）
- **THEN** 聊天区顶栏 SHALL 出现轻量「轨迹」入口，点击 SHALL 跳转到 `/agent-runs/:id`

#### Scenario: Per-conversation entry hidden for empty draft
- **WHEN** 用户处于空草稿状态（`conversations.activeId` 为空）或当前会话无消息
- **THEN** 「轨迹」入口 MUST NOT 出现于聊天首页

#### Scenario: No agent-runs list page
- **WHEN** 用户访问 `/agent-runs`（无会话 id）
- **THEN** 应用 MUST 重定向到聊天首页 `/`；MUST NOT 渲染跨会话 Run 列表

#### Scenario: Detail back returns to conversation only
- **WHEN** 用户在 `/agent-runs/:id` 点击返回
- **THEN** 应用打开该会话并进入聊天首页 `/`；MUST NOT 提供「返回列表」入口

#### Scenario: Click record opens inspector
- **WHEN** 用户点击账本中一条模型或工具记录
- **THEN** 右侧打开检查器：助手记录可切换概述/思考/输出/计时；工具记录可切换参数/结果/计时；再点同一行或关闭按钮收起检查器

#### Scenario: Fold state restored on re-enter
- **WHEN** 用户在某个会话轨迹详情手动收起轮次、收起调用或选中某行后退出，再重新进入该详情
- **THEN** 重新进入时轮次全部展开、工具调用可见、无选中行

#### Scenario: Record index shown
- **WHEN** 用户打开含 2 条及以上记录的会话轨迹详情
- **THEN** 每条记录显示其在会话内的连续序号 `#N`（从 1 起，跨轮次连续）

#### Scenario: Cascade delete removes trajectory
- **WHEN** 用户删除某个会话（既有的 `delete_conversation` 流程）
- **THEN** 访问已被删除的 `/agent-runs/:id` MUST 显示错误或空态提示，不暴露残留轨迹

#### Scenario: Sidebar visible on agent-run detail
- **WHEN** 路由为 `/agent-runs/:id`
- **THEN** 侧边栏 SHALL 可见；详情页顶部 SHALL 提供「返回会话」入口（回到该会话聊天）

### Requirement: Agent Run Trajectory Export

详情页 SHALL 提供「一键导出」与「复制为 Markdown」两个动作，用于把当前 Run 的**完整交互事件集合**带出应用供外部排查（导出粒度 = 一个 Run 的全部账本记录）。

**Markdown 导出** MUST 包含：Run 标题、id、更新时间、消息总数、轮次数、助手记录数、工具调用数；正文按账本分组：
- `## 第 N 轮`（独立压缩用 `## 轮次之间`）
- `### 消息` / `### 第 M 步`
- `#### #N 用户|系统|助手|工具 · <tool_id>|上下文|已压缩` 段（模型含思考引用块与输出；工具含 Args/Result 折叠代码块；提示词含全文折叠块）

**JSON 导出** MUST 包含：`{ run, messages, sessionEvents, meta }` —— `sessionEvents` 为原始日志；`messages.trajectory_json` 保留扁平 `TrajectoryEvent[]`；MUST NOT 包含密钥或环境变量。

**复制为 Markdown** SHALL 把与「导出 Markdown」同一字符串写入系统剪贴板并显示成功反馈。**下载** SHALL 复用 `downloadTextFile()` 触发 WebView 内置下载，文件名 SHALL 形如 `<sanitized-title>-<id前8位>-<YYYYMMDD>.md|.json`。MUST NOT 弹出原生保存对话框或写入应用外路径。

#### Scenario: Export markdown downloads file with turn sections
- **WHEN** 用户在 `/agent-runs/:id` 点击「导出 Markdown」且该 Run 含多个交互事件
- **THEN** 浏览器触发下载，文件名匹配 `<title>-<id前8>-<YYYYMMDD>.md`，内容按「第 N 轮 / 消息 / 第 M 步」组织，记录标题为 `用户` / `系统` / `助手` / `工具` / `上下文`

#### Scenario: Export json downloads structured file
- **WHEN** 用户在 `/agent-runs/:id` 点击「导出 JSON」
- **THEN** 浏览器触发下载，文件名匹配 `<title>-<id前8>-<YYYYMMDD>.json`，内容含 `run` / `messages` / `sessionEvents`、不带任何敏感凭据

#### Scenario: Copy markdown puts text on clipboard
- **WHEN** 用户在 `/agent-runs/:id` 点击「复制为 Markdown」
- **THEN** 剪贴板内出现与「导出 Markdown」相同的字符串，UI 给出复制成功反馈（如按钮文案短暂切换为「已复制」）

#### Scenario: Export unavailable on empty run
- **WHEN** 用户打开的 Run 没有消息（空草稿被持久化前的状态不应进入该页；若意外进入）
- **THEN** 详情页显示空态文案，导出与复制按钮 SHALL 处于禁用态

#### Scenario: No cloud upload on export
- **WHEN** 用户触发任意导出或复制动作
- **THEN** 浏览器 DevTools Network / Tauri 日志中 MUST NOT 出现向任何外部域名发起的请求（导出完全在本地完成）

### Requirement: Chat Code Block Rendering

聊天页面的 Markdown 渲染 SHALL 对 fenced 代码块提供美化展示，MUST 覆盖日常开发格式：`json`、`yaml`、`toml`、`markdown`、`bash/shell`、`sql`、`xml/html`、`ini`、常见编程语言（js/ts/py/go/rust/java/c/cpp 等，由 highlight.js common 集提供）。每个代码块 MUST 展示：语言标签（右上角）、语法高亮着色（配 highlight.js 主题样式，深色底）、圆角卡片容器、横向滚动（不换行溢出）、**复制按钮（左上角，hover 代码块时出现，点击复制代码原文并给出成功反馈）**；未知语言标签 MUST 优雅降级为纯等宽文本（不崩溃、不空白）。流式输出中未闭合的代码块 MUST 同样安全渲染。渲染输出 MUST 继续经过 DOMPurify 消毒（复制按钮 MUST NOT 使用内联事件，经全局事件委托实现）。

#### Scenario: Json code block highlighted
- **WHEN** 助手回复包含 ```json fenced 代码块
- **THEN** 代码块以深色卡片展示，键/字符串/数字按主题着色，右上角显示 JSON 语言标签，内容横向可滚动

#### Scenario: Yaml and toml blocks highlighted
- **WHEN** 助手回复包含 ```yaml 与 ```toml 代码块
- **THEN** 两种格式均正确识别并语法高亮展示（toml 可经 ini 别名识别），带语言标签

#### Scenario: Unknown language degrades gracefully
- **WHEN** 代码块语言标签为未注册的自定义语言（如 ```mylang）
- **THEN** 该块以纯等宽文本展示（无着色但不空白、不报错），语言标签原样显示

#### Scenario: Code block copy button copies raw code
- **WHEN** 用户 hover 一个代码块并点击左上角复制按钮
- **THEN** 该代码块的**原始代码文本**（未高亮 HTML）被复制到剪贴板，按钮短暂切换为成功态（勾号）

#### Scenario: Streaming renders open code fence safely
- **WHEN** 流式输出中出现未闭合的 ``` 代码块
- **THEN** 已到达内容按代码块安全渲染（消毒后），后续 token 到达时增量更新，不闪烁崩溃

## MODIFIED Requirements

### Requirement: Assistant Turn Trajectory Display

聊天界面 SHALL 将同一助手回合内的思考、工具调用与正文按发生顺序渲染为一条时间线（轨迹）。**过程轨（思考 + 工具）默认收起为一行摘要**（如「N 次工具调用 · M 条消息」+ chevron）；用户点击摘要后展开为左侧贯通竖线的紧凑账本；助手正文（最终回复）MUST 始终可见、不随过程轨折叠。流式进行中过程轨 MUST 默认展开并随事件增量更新；回合结束后（及历史重开）过程轨 MUST 恢复为收起摘要。展开后单步思考/工具详情默认再收起，可逐条点开；重开会话后 MUST 恢复上述默认折叠规则。底层 trajectory 数据以扁平 `TrajectoryEvent[]` 持久化；聊天首页 MUST 通过读侧降级（flatten）以既有 flat 形态展示。

#### Scenario: Streaming turn shows ordered steps expanded
- **WHEN** 一轮包含思考与工具调用的回复正在流式生成
- **THEN** 过程轨默认展开，按顺序展示思考、工具（参数与结果状态）、正文，且各步骤随事件更新

#### Scenario: Completed turn collapses process to summary
- **WHEN** 该回合完成或用户重新打开含轨迹的历史会话，且该回合含思考或工具步骤
- **THEN** 过程轨默认收起为摘要行（含工具次数与消息条数）；助手正文保持展开可见；点击摘要行后展开过程账本

#### Scenario: Manual expand does not persist across reopen
- **WHEN** 用户在已完成回合中手动展开过程轨或某思考/工具步骤后关闭并重新打开该会话
- **THEN** 再次以默认收起状态展示（过程轨收起为摘要）

#### Scenario: Chat rendering unchanged by structured persistence
- **WHEN** 后端将 trajectory 以扁平 `TrajectoryEvent[]` 持久化后，用户在聊天首页查看该回合
- **THEN** 聊天首页仍以既有 flat 时间线（思考/工具/正文）展示，另叠加过程轨摘要折叠

### Requirement: Trajectory Persistence

带过程步骤的助手消息 SHALL 将有序扁平 `TrajectoryEvent[]` 持久化到本地 SQLite（`messages.trajectory_json`）：`model_call`（一次 agent↔model 调用，含 `id` / `reasoning?` / `requested_tools` / `response?` / `stream_mode?`）与 `tool_call`（一次 agent↔tool 调度，含 `id` / `args` / `result?` / `status`）依次排列，每次交互一条、无嵌套。历史会话重开时 MUST 按原顺序恢复事件列表。已存在的数据库 MUST 通过向后兼容的方式处理旧数据——旧消息无 `trajectory_json` 时 MUST 由 `reasoning` / `tool_calls_json` / `content` 合成一条降级时间线；旧消息的 `trajectory_json` 为 legacy `[{type:"reasoning"|"tool"|"text"}]` 数组时 MUST 由 `resolve_trajectory` 按启发式转换（每个 reasoning 开启新 model_call、tools 挂其后、末尾 text 为最终回复）；开发期嵌套格式（model_call 内嵌 tools 数组）MUST 在读侧拍平为扁平事件。

#### Scenario: Reopen restores event order
- **WHEN** 用户重新打开一条含多个交互事件的历史会话
- **THEN** 事件顺序与生成时一致（model_call 在前、其 tool_call 紧随其后），工具调用可见而不丢失

#### Scenario: Legacy message without trajectory
- **WHEN** 旧库消息仅有 content/reasoning/tool_calls 而无轨迹字段
- **THEN** 界面仍展示降级时间线，应用不崩溃

#### Scenario: Legacy flat-array trajectory converts via heuristic
- **WHEN** 旧库消息 `trajectory_json` 为 legacy flat `[{type:"reasoning"|"tool"|"text"}]` 数组
- **THEN** 读侧按启发式转换为扁平事件（reasoning 开启新调用、tools 挂其后、末尾 text 为最终回复），工具调用与正文均可见

#### Scenario: New persistence writes flat event array
- **WHEN** Agent 循环完成一次助手回合且产生了至少两次模型调用与一次工具调用
- **THEN** `messages.trajectory_json` MUST 为扁平 `[{type:"model_call",...},{type:"tool_call",...},...]`，model_call 事件数等于该回合的模型调用次数、tool_call 事件数等于工具调用次数
