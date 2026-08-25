# Proposal: chat-message-experience

## Why

聊天首页（`src/views/HomePage.vue`）当前把整个页面当作滚动容器：消息变多时整窗滚动，输入框与顶栏被推走；助手回复纯文本展示、无法复制；工具调用卡片信息密度低；模型思考过程完全没有展示。作为应用主入口的对话体验明显落后于产品定位，需要一次聚焦的消息区体验升级。

## What Changes

- **消息区滚动收敛**：仅消息列表内部滚动（`overflow-y: auto` + 固定可视高度），顶栏 / LLM 提示条 / 输入框固定不被滚动；流式输出时自动滚动到底部（用户手动上滚则暂停跟随）。
- **思考过程展示（新能力，全链路）**：
  - Rust：`ChatModel` 返回 `reasoning`（genai 后端读取 reasoning content，embedded 引擎尽力捕获思考 token）；`AgentEvent` 新增 `reasoning` 事件；`messages` 表新增 `reasoning` 列持久化。
  - 前端：助手消息上方展示「思考过程」折叠块，**默认不展开**，可点击展开/收起；流式期间显示思考中状态。
- **Markdown 渲染**：助手消息与思考过程以 Markdown 渲染（代码块语法高亮、表格、列表等），新引入 `markdown-it` + `highlight.js` + `DOMPurify`（XSS 防护）；用户消息保持纯文本；流式期间安全渲染。
- **消息可复制**：每条用户/助手消息 hover 出现复制按钮，点击将**原始 Markdown 文本**复制到剪贴板并给成功反馈。
- **消息 UI 重做**：用户/助手消息气泡、流式指示、错误条、工具调用卡片（名称图标、参数、结果、状态）统一视觉规范（间距、圆角、等宽字体、暗色适配）。

## Capabilities

### New Capabilities
- `chat-reasoning-display`: 思考过程的捕获、持久化、事件下发与前端折叠展示（默认收起）。

### Modified Capabilities
- `agent-chat`: 消息区滚动行为收敛为仅消息列表滚动 + 自动跟随；消息支持一键复制；助手消息 Markdown 渲染（高亮 + XSS 防护）；消息/工具/错误/流式各状态的 UI 规范更新。

## Impact

- 前端：`src/views/HomePage.vue`（重写模板与样式）、`src/stores/conversations.ts`（`ChatMessage` 增加 `reasoning` 字段）；新增依赖 `markdown-it`、`highlight.js`、`dompurify`（+ `@types/*`）。
- Rust：`src-tauri/src/agent/loop.rs`（`AgentEvent::Reasoning`、持久化）、`src-tauri/src/agent/llm.rs`（`ModelTurn` 携带 reasoning）、`src-tauri/src/agent/genai_model.rs`、`src-tauri/src/agent/embedded_engine.rs`、`src-tauri/src/commands/agent.rs`（事件透传）、`src-tauri/src/commands/conversation.rs` + DB migration（`reasoning` 列）。
- 不新增工具 id，不涉及工具注册表。

## Non-goals

- 不做 Markdown 编辑器预览（仅渲染展示）；不做 KaTeX 数学公式与 Mermaid 图表（后续可加）。
- 不做多模型思考预算 / 开关设置项；思考过程有则展示，无则不显示区块。
- 不改会话历史侧栏（`SideBar.vue`）交互。
- 不引入流式 token 级 API 改造（维持现有事件机制，仅新增事件类型）。
