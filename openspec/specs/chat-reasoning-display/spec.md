# chat-reasoning-display

## Purpose

模型思考/推理内容的捕获、持久化、事件下发与前端折叠展示（默认收起）。该 capability 覆盖从模型后端返回的思考内容到前端折叠块展示的完整链路。

## Requirements



### Requirement: Reasoning Capture from Model Backends
当模型后端返回思考/推理内容时，系统 SHALL 在 Rust 侧捕获该内容并与正文区分：`ChatModel` 的返回 MUST 区分 `reasoning`（思考过程）与正文内容；后端不提供思考内容时 `reasoning` MUST 为空而不是报错。

#### Scenario: Backend provides reasoning
- **WHEN** 所配置的 LLM 后端在回复中附带 reasoning content（如 DeepSeek 类模型的思考输出）
- **THEN** agent 循环拿到与正文分离的思考文本，且不把它混入助手正文

#### Scenario: Backend has no reasoning
- **WHEN** 后端回复不包含任何思考内容
- **THEN** 该轮回复 `reasoning` 为空，行为与现状一致，不产生错误

### Requirement: Reasoning Event Streaming
Agent 循环 SHALL 通过 `agent-event` 事件向前端下发思考内容（`reasoning` 事件类型），前端在流式期间能区分「思考中」与「正文输出中」两种阶段。思考增量 MUST 作为助手回合时间线中的思考步骤展示（见 agent-chat 轨迹要求），不得仅在有正文前短暂显示标题而无可展开内容。

#### Scenario: Reasoning streamed during turn
- **WHEN** 一轮包含思考内容的回复正在生成
- **THEN** 前端先收到或交错收到 `reasoning` 事件（思考文本），并与随后的 `token` 事件区分；时间线中思考步骤内容随增量可见

#### Scenario: Turn without reasoning
- **WHEN** 一轮回复没有思考内容
- **THEN** 前端只收到 `token` 等既有事件，不收到空的 `reasoning` 事件，也不渲染空思考步骤

### Requirement: Reasoning Persistence
带思考内容的助手消息 SHALL 将思考文本持久化到本地 SQLite（`messages` 新增 `reasoning` 列）；历史会话重新打开时思考过程 MUST 能再次展示。已存在的数据库 MUST 通过向后兼容的 migration 自动加列，旧数据不丢失。

#### Scenario: Reopen conversation with reasoning
- **WHEN** 用户重新打开一条包含思考过程的历史会话
- **THEN** 助手消息仍能展示其思考过程（默认收起）

#### Scenario: Existing database upgraded
- **WHEN** 应用在已有 `messages` 表的旧数据库上启动
- **THEN** migration 补齐 `reasoning` 列，旧消息可正常读取且 `reasoning` 为空

### Requirement: Collapsed-by-Default Reasoning Display
前端 SHALL 在助手回合时间线中展示「思考过程」步骤；模型未提供思考内容时 MUST NOT 渲染该步骤。流式进行中思考步骤 MUST 默认展开；回合结束后与历史重开时 MUST 默认收起，点击标题 MUST 可切换展开/收起。

#### Scenario: Reasoning expanded while streaming
- **WHEN** 思考内容正在流式到达且回合尚未结束
- **THEN** 思考步骤默认展开并显示已到达的思考文本

#### Scenario: Reasoning collapsed by default
- **WHEN** 一条带思考内容的助手消息渲染完成
- **THEN** 时间线中出现「思考过程」步骤，内容默认不展开，仅显示摘要标题

#### Scenario: Toggle reasoning visibility
- **WHEN** 用户点击思考步骤标题
- **THEN** 思考内容在展开与收起之间切换，再次点击恢复

#### Scenario: No reasoning no block
- **WHEN** 助手消息没有思考内容
- **THEN** 不出现「思考过程」步骤

### Requirement: Think-Tag Extraction
对模型输出中内联的思考标记（`<think>…</think>`，含被截断未闭合的 `<think>…`），系统 SHALL 在回合收尾前将其从正文剥离并作为 reasoning 事件/字段单独输出，MUST NOT 把思考标签或其内容渲染进助手正文。后端已分离的 reasoning（如 `reasoning_content`、Anthropic `thinking` 块）与内联 think 标签并存时 SHALL 合并输出。无思考内容的回复行为不变。

#### Scenario: Think tag stripped from body
- **WHEN** 本地/云端模型输出 `<think>推理…</think>正文`
- **THEN** 思考内容经 reasoning 事件单独流式展示（默认收起），正文不含 `<think>` 标签与思考内容

#### Scenario: Unclosed think tag at truncation
- **WHEN** 生成被截断，输出仅有 `<think>推理…` 而无闭合标签
- **THEN** 思考内容仍被剥离到 reasoning，不泄漏到正文

### Requirement: Chunked Streaming Emission
Agent 循环 SHALL 将思考与正文以增量事件流式发出。真流式后端 MUST 在生成过程中发出增量；降级路径 MUST 将完整文本分块经 `reasoning` / `token` 发出（而非整段一次性单事件），前端流式渲染与自动跟随行为保持可用。本要求不排除在 `fallback` 模式下使用分块策略。

#### Scenario: Tokens arrive in chunks
- **WHEN** 模型回合输出多段文本（真流式或降级分块）
- **THEN** 前端在回合收尾前持续接收多个 token/reasoning 事件并逐步渲染

#### Scenario: Fallback still chunked
- **WHEN** 后端以 fallback 模式完成整段生成后下发文本
- **THEN** 文本仍经多个分块事件到达前端，而不是单个巨包事件导致界面一次性跳变（空文本除外）

### Requirement: True Streaming Preferential Emission
模型后端在支持流式时 SHALL 在生成过程中增量发出 `reasoning` / `token`（或等价 delta）事件，而不是仅在整段生成结束后再切块。Agent 循环 MUST 优先消费流式接口；当协议或客户端无法真流式时 MUST 降级为整段完成后的分块发出，并配合 `stream_meta.mode=fallback`（见 agent-chat）。半截工具调用 JSON MUST NOT 被调度执行；完整 `tool_calls` 回合结束后再进入既有校验与 dispatch。

#### Scenario: Live deltas during generation
- **WHEN** 支持流式的后端（如本地嵌入引擎或 OpenAI 兼容 SSE）正在生成长回复
- **THEN** 前端在生成完成前持续收到多个增量 `token` 和/或 `reasoning` 事件并逐步渲染

#### Scenario: Unsupported protocol falls back
- **WHEN** 当前协议无法提供真流式
- **THEN** 系统在发出文本前下发 `fallback` 元数据，再以分块方式发出完整思考与正文，行为对用户仍为逐步渲染且不报错

#### Scenario: Partial tool JSON not executed
- **WHEN** 流式过程中工具调用参数尚未形成完整可解析调用
- **THEN** 系统不执行该工具，待本轮模型输出完整后再按既有 Agent 循环处理
