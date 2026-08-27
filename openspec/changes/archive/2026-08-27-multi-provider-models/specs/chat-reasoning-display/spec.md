## ADDED Requirements

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
