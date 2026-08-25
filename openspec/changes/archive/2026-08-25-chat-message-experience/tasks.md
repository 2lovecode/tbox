# Tasks: chat-message-experience

## 1. 后端 reasoning 通道

- [x] 1.1 `src-tauri/src/agent/llm.rs`：`ModelTurn::Text` 增加 `reasoning` 字段（默认空），更新既有测试构造
- [x] 1.2 `src-tauri/src/agent/genai_model.rs`：开启 `capture_reasoning_content`，从 `chat_res.reasoning_content` 填充 reasoning
- [x] 1.3 `src-tauri/src/agent/embedded_engine.rs`：若思考 token 可分离则填充 reasoning，否则留空（不阻塞）
- [x] 1.4 `src-tauri/src/agent/loop.rs`：`AgentEvent` 新增 `Reasoning { text }`，Text 分支在 `Token` 前 emit；带 reasoning 时持久化写入
- [x] 1.5 `cargo test` 全绿

## 2. 持久化与事件透传

- [x] 2.1 `src-tauri/src/commands/conversation.rs`：建表加 `reasoning` 列；启动 migration `ALTER TABLE` 幂等加列（忽略 duplicate column），附单测
- [x] 2.2 `append_assistant_message_on` / `get_messages_on` / `ChatMessage` serde 结构携带 `reasoning`
- [x] 2.3 `src-tauri/src/commands/agent.rs`：`AgentEventPayload` 透传 `reasoning` 事件类型

## 3. 前端数据层

- [x] 3.1 `src/stores/conversations.ts`：`ChatMessage` 增加 `reasoning?: string`
- [x] 3.2 `HomePage.vue` 事件处理新增 `reasoning` 分支（累积 `streamingReasoning`），`finalizeStreaming` 写入消息并随 reload 对齐

## 4. 消息区布局与滚动

- [x] 4.1 `.chat-home` 改为固定视口 flex 布局，`.message-list` 唯一滚动容器（`flex:1; overflow-y:auto; min-height:0`），窗口整体不滚动
- [x] 4.2 自动跟随：scroll 监听 + 贴底阈值（<40px），流式输出仅贴底时滚底，用户上滚暂停、回底恢复

## 5. 消息 UI 与交互

- [x] 5.1 思考过程折叠块：默认收起，点击切换展开/收起；无 reasoning 不渲染；流式期间显示思考中状态
- [x] 5.2 复制按钮：每条用户/助手消息 hover/聚焦出现，复制原始 Markdown 源文本并给成功反馈（✓ 1.5s）
- [x] 5.3 Markdown 渲染：安装 `markdown-it` + `highlight.js` + `dompurify`（+ `@types/*`），封装渲染模块（单例 md 实例、按需高亮语言、DOMPurify 消毒、外链 noopener），助手消息与思考过程使用，用户消息保持纯文本
- [x] 5.4 Markdown 样式：代码块语言标签、表格、列表、引用、行内代码样式，暗色适配
- [x] 5.5 消息气泡重做：用户右对齐主色气泡、助手左对齐卡片，流式输出中指示动画
- [x] 5.6 工具卡片重做：名称+图标+状态徽标（运行中/完成/失败），参数/结果等宽字体限高内部滚动
- [x] 5.7 错误条固定于输入区上方，不随消息滚动隐藏

## 6. 验证

- [x] 6.1 `pnpm build`（或 typecheck）+ `cargo test` 通过
- [ ] 6.2 手动验证：长会话仅消息区滚动、流式跟随/暂停、思考默认收起可展开、复制成功、Markdown/代码块/表格渲染正常、`<script>` 注入被消毒、工具卡片与错误条展示
