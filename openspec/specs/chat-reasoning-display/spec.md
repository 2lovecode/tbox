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
Agent 循环 SHALL 通过 `agent-event` 事件向前端下发思考内容（新增 `reasoning` 事件类型），前端在流式期间能区分「思考中」与「正文输出中」两种阶段。

#### Scenario: Reasoning streamed during turn
- **WHEN** 一轮包含思考内容的回复正在生成
- **THEN** 前端先收到 `reasoning` 事件（思考文本），随后收到 `token` 事件（正文）

#### Scenario: Turn without reasoning
- **WHEN** 一轮回复没有思考内容
- **THEN** 前端只收到 `token` 等既有事件，不收到空的 `reasoning` 事件

### Requirement: Reasoning Persistence
带思考内容的助手消息 SHALL 将思考文本持久化到本地 SQLite（`messages` 新增 `reasoning` 列）；历史会话重新打开时思考过程 MUST 能再次展示。已存在的数据库 MUST 通过向后兼容的 migration 自动加列，旧数据不丢失。

#### Scenario: Reopen conversation with reasoning
- **WHEN** 用户重新打开一条包含思考过程的历史会话
- **THEN** 助手消息仍能展示其思考过程（默认收起）

#### Scenario: Existing database upgraded
- **WHEN** 应用在已有 `messages` 表的旧数据库上启动
- **THEN** migration 补齐 `reasoning` 列，旧消息可正常读取且 `reasoning` 为空

### Requirement: Collapsed-by-Default Reasoning Display
前端 SHALL 在助手消息上方展示「思考过程」折叠块，默认收起；点击标题 MUST 可切换展开/收起。模型未提供思考内容时 MUST NOT 渲染该区块。

#### Scenario: Reasoning collapsed by default
- **WHEN** 一条带思考内容的助手消息渲染完成
- **THEN** 消息上方出现「思考过程」折叠块，内容默认不展开，仅显示摘要标题

#### Scenario: Toggle reasoning visibility
- **WHEN** 用户点击折叠块标题
- **THEN** 思考内容在展开与收起之间切换，再次点击恢复

#### Scenario: No reasoning no block
- **WHEN** 助手消息没有思考内容
- **THEN** 不出现「思考过程」区块

### Requirement: Think-Tag Extraction
对模型输出中内联的思考标记（`<think>…</think>`，含被截断未闭合的 `<think>…`），系统 SHALL 在回合收尾前将其从正文剥离并作为 reasoning 事件/字段单独输出，MUST NOT 把思考标签或其内容渲染进助手正文。后端已分离的 reasoning（如 `reasoning_content`、Anthropic `thinking` 块）与内联 think 标签并存时 SHALL 合并输出。无思考内容的回复行为不变。

#### Scenario: Think tag stripped from body
- **WHEN** 本地/云端模型输出 `<think>推理…</think>正文`
- **THEN** 思考内容经 reasoning 事件单独流式展示（默认收起），正文不含 `<think>` 标签与思考内容

#### Scenario: Unclosed think tag at truncation
- **WHEN** 生成被截断，输出仅有 `<think>推理…` 而无闭合标签
- **THEN** 思考内容仍被剥离到 reasoning，不泄漏到正文

### Requirement: Chunked Streaming Emission
Agent 循环 SHALL 将最终文本与思考内容以分块方式经 token / reasoning 事件流式发出（而非整段一次性发出），前端现有流式渲染与自动跟随行为保持不变。

#### Scenario: Tokens arrive in chunks
- **WHEN** 模型回合完成并输出多段文本
- **THEN** 前端在回合收尾前持续接收多个 token/reasoning 事件并逐步渲染
