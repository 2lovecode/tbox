# agent-chat Delta

## ADDED Requirements

### Requirement: Message List Scroll Containment
聊天首页 SHALL 将滚动收敛到消息列表内部：顶栏、LLM 提示条与输入框 MUST 固定可视，不随消息滚动；页面整体（窗口）MUST NOT 出现滚动条。

#### Scenario: Long conversation scrolls only messages
- **WHEN** 当前会话消息高度超出可视区域
- **THEN** 仅消息列表内部出现滚动条并可滚动，输入框与顶部区域保持固定

#### Scenario: Auto-follow streaming output
- **WHEN** 流式回复输出且用户当前停在消息列表底部附近
- **THEN** 消息列表自动滚动跟随新内容

#### Scenario: User scroll pauses auto-follow
- **WHEN** 用户在流式输出期间向上滚动离开底部
- **THEN** 自动跟随暂停，不抢夺用户滚动位置；用户滚回底部后恢复跟随

### Requirement: Message Copy
系统 SHALL 为每条用户与助手消息提供复制操作：鼠标悬停（或聚焦）消息时出现复制按钮，点击后将该消息**原始 Markdown 文本**（未渲染源文本）复制到系统剪贴板，并给出成功反馈。

#### Scenario: Copy assistant message
- **WHEN** 用户 hover 一条助手消息并点击复制按钮
- **THEN** 该消息的原始 Markdown 源文本被复制到剪贴板，按钮给出复制成功反馈

#### Scenario: Copy user message
- **WHEN** 用户 hover 一条自己发出的消息并点击复制按钮
- **THEN** 该消息文本被复制到剪贴板，按钮给出复制成功反馈

### Requirement: Markdown Rendering
助手消息与思考过程内容 SHALL 以 Markdown 渲染展示，MUST 支持：代码块（带语言标签与语法高亮）、行内代码、表格、列表、链接、引用、粗体/斜体。渲染输出 MUST 经 HTML 消毒（防 XSS），外链 MUST NOT 自动执行脚本。用户消息 MUST 保持纯文本展示。流式输出期间 MUST 同样安全渲染未完成的 Markdown。

#### Scenario: Assistant markdown rendered
- **WHEN** 助手消息包含 fenced 代码块、表格与列表
- **THEN** 消息按 Markdown 渲染展示，代码块带语言标签与语法高亮

#### Scenario: Malicious HTML sanitized
- **WHEN** 模型输出包含 `<script>` 或 `onerror` 等注入 HTML
- **THEN** 渲染结果中脚本与事件处理器被消毒移除，不执行任何注入代码

#### Scenario: User message stays plain text
- **WHEN** 用户消息包含 Markdown 记号（如 `#` 或反引号）
- **THEN** 该消息按纯文本原样展示，不做 Markdown 渲染

### Requirement: Message Visual Hierarchy
消息区 UI SHALL 统一视觉规范：用户/助手气泡区分对齐与配色，助手消息支持流式输出指示（输出中状态），工具调用卡片 MUST 展示工具名、状态（运行中/完成/失败）、参数与结果（等宽字体、可滚动、限高折叠），错误提示条 MUST 与消息流视觉区分且不遮挡输入区。

#### Scenario: Streaming assistant indicator
- **WHEN** 助手回复正在流式输出
- **THEN** 消息区呈现输出中状态指示，结束后指示消失

#### Scenario: Tool call card presentation
- **WHEN** 一轮对话包含工具调用
- **THEN** 工具卡片按顺序展示工具名、状态徽标、参数与结果，参数/结果以等宽字体限高展示并可滚动

#### Scenario: Error bar stays visible
- **WHEN** 本轮出现错误提示
- **THEN** 错误条在输入区上方可见，不随消息滚动隐藏，不遮挡输入框
