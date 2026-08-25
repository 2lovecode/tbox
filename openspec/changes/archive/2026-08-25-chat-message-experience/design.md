# Design: chat-message-experience

## Context

聊天首页 `src/views/HomePage.vue` 是应用根路由，当前布局为 `min-height: calc(100vh - 180px)` 的纵向流：消息多时整个窗口滚动。后端 `run_agent`（`src-tauri/src/agent/loop.rs`）以非流式 `exec_chat` + 事件转发的方式工作，`ModelTurn` 只有正文与 tool calls，思考内容在 genai 层（`ChatResponse.reasoning_content`，需 `capture_reasoning_content` 开启）与 embedded 引擎处均未捕获。`messages` 表无 reasoning 列。

## Goals / Non-Goals

**Goals:**
- 消息列表成为唯一滚动容器；顶栏/提示条/输入框固定。
- 思考过程：捕获 → 事件 → 持久化 → 前端折叠展示（默认收起）。
- 每条消息可复制；消息/工具/错误/流式 UI 统一。

**Non-Goals:**
- Markdown 渲染、token 级流式改造、思考开关设置、侧栏改版（见 proposal Non-goals）。

## Decisions

1. **布局改为 flex 固定视口**：`.chat-home` 用 `height: 100%; display:flex; flex-direction:column; overflow:hidden`，`.message-list` 设 `flex:1; overflow-y:auto; min-height:0`。选择它而非 JS 计算高度，因为纯 CSS 即可保证「窗口永不滚动」。外层容器（`App.vue` 主区域）需确认无多余滚动。
   - 自动跟随：监听列表 `scroll`，距底 < 40px 视为「贴底」；`watch` 流式文本/消息数组时仅贴底才 `scrollTop = scrollHeight`。

2. **reasoning 通道（Rust）**：
   - `llm.rs`：`ModelTurn::Text { text, reasoning }`（新增字段，`#[serde(default)]`/`Option` 兼容旧测试）。
   - `genai_model.rs`：`ChatOptions::capture_reasoning_content = true`，从 `chat_res.reasoning_content` 读取。
   - `embedded_engine.rs`：尽力捕获（GGUF 思考 token 若可分离则填入，否则为空），不阻塞主线。
   - `loop.rs`：`AgentEvent` 新增 `Reasoning { text }`，在 `Token` 之前 emit；持久化时写入新列。

3. **持久化 migration**：`conversation.rs` 建表语句加 `reasoning TEXT NOT NULL DEFAULT ''`，启动时 `ALTER TABLE messages ADD COLUMN` + 忽略「duplicate column」错误（沿用现有 migration 风格，避免引入版本表）。备选 user_version 方案被否：改动面更大且现库尚小。

4. **前端数据流**：`ChatMessage` 增加 `reasoning?: string`；`HomePage.vue` 事件处理新增 `reasoning` 分支累积到 `streamingReasoning`，`finalizeStreaming` 写入本地消息（随后仍由 reload 对齐持久化 id）。折叠状态按消息 id 存 `Set`，流式期间新消息默认收起。

5. **复制**：复用项目现有 `CopyButton.vue` 模式或 `navigator.clipboard.writeText`；复制**原始 Markdown 源文本**（渲染后 DOM 取不到源码，因此直接用 store 里的 `content`）；hover 显示按钮（触屏/键盘聚焦也可见），成功后按钮图标切换为 ✓ 约 1.5s。

6. **Markdown 渲染**：新依赖 `markdown-it` + `highlight.js` + `dompurify`。
   - 渲染封装为独立工具模块（`src/utils/markdown.ts` 或轻量组件 `MessageMarkdown.vue`）：`markdown-it` 实例单例，`highlight.js` 按需注册常用语言，`linkify` 开启；渲染结果经 `DOMPurify.sanitize` 后 `v-html` 插入。
   - 选 markdown-it 而非 marked：插件生态与 CommonMark 兼容更好、默认安全配置成熟；选 highlight.js 而非 shiki：体积小、同步渲染、无需异步 WASM。
   - 流式期间每次事件都对全文重新渲染（当前模型回复为一次性文本，量级可控）；后端真流式化后再考虑增量渲染（Non-goal）。
   - 用户消息不做 Markdown（spec 定义），避免误渲染用户输入的记号。
   - 暗色适配：代码高亮主题跟随现有暗色方案切换（如可用 CSS 变量）。

7. **UI 重做**：保持现有配色变量（`--primary` 等）；用户气泡右侧主色、助手左侧白底卡片（暗色模式用 CSS 变量）；工具卡片头（图标+名称+状态徽标）常显，参数/结果等宽字体 `max-height` 限高内部滚动；流式指示用打字点动画。

## Risks / Trade-offs

- [genai 0.6 部分适配器不回 reasoning] → 空则不展示区块，spec 已定义该场景。
- [ALTER TABLE 幂等] → 捕获 duplicate column 错误；补一条 Rust 单测。
- [自动跟随与用户抢滚动] → 贴底阈值 + scroll 监听暂停策略。
- [非流式下「思考中」阶段感弱] → `Reasoning` 事件先于 `Token` 到达即可展示，embedded/云端实际表现后续可升级为真流式（Non-goal）。
- [v-html XSS 面] → 全部经 DOMPurify 消毒 + spec 场景用例覆盖；链接默认 `target=_blank rel=noopener`。
- [流式全量重渲染性能] → 消息体量小可接受；真流式化后再做增量。

## Migration Plan

后端先行（事件 + 列），前端随后；DB migration 向后兼容，回滚仅需还原代码，新增列不影响旧代码读取。

## Open Questions

- embedded 引擎的思考 token 能否稳定分离（实现期验证，不稳定则该后端 reasoning 留空）。
