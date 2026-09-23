## 1. Schema & Rename API

- [x] 1.1 在 `ensure_conversation_schema` 中为 `conversations` 增加 `title_locked INTEGER NOT NULL DEFAULT 0`（懒迁移）
- [x] 1.2 实现 `rename_conversation(id, title)`：校验非空/长度、trim、写库、`title_locked=1`、更新 `updated_at`；在 `lib.rs` 注册
- [x] 1.3 前端 store 增加 `renameConversation`；侧栏点击标题可编辑或依赖头栏编辑后列表刷新

## 2. Title Summarizer（独立 completion）

- [x] 2.1 新增 `title_summarizer` 模块：自有短 system+user、单次 `complete`、输出清洗；复用 active LLM 配置解析；MUST NOT 拼装对话系统提示词 / Skill / 工具；Mock 返回可预期假标题
- [x] 2.2 新建会话首条 `append_user_message` 成功后 spawn 异步任务调用 Summarizer：成功且 `title_locked=0` 时 UPDATE title，并 `emit("conversation:title", { id, title })`
- [x] 2.3 单测：截断占位、解析清洗、locked 不覆盖；断言 Summarizer 消息不含工具/Skill 系统块；Mock 下总结可完成

## 3. 聊天顶栏 UI

- [x] 3.1 `HomePage` 增加 `chat-chrome`：左标题（草稿显示「新对话」）、右轨迹入口；移除绝对定位浮层轨迹按钮
- [x] 3.2 标题就地编辑：Enter/失焦调用 rename；监听 `conversation:title` 更新 store 与展示
- [x] 3.3 草稿态禁用轨迹入口；已持久化会话可进 `/agent-runs/:id`

## 4. 验收

- [ ] 4.1 手动：新会话首条消息 → 侧栏先截断标题 → 稍后变为 LLM 短标题；中途手动改名不被覆盖
- [ ] 4.2 手动：顶栏标题与轨迹入口布局正确；浅色/深色与现有壳层风格一致
- [x] 4.3 `cargo test` 相关 conversation/title 测试通过；前端无新增类型错误
