## Context

会话创建时 `append_user_message` 用 `truncate_title`（前 40 字）写入 `conversations.title`。前端侧栏展示该字段，聊天页无顶栏；轨迹入口为 `HomePage` 内绝对定位按钮。用户选择用当前 LLM **经独立 Title Summarizer**异步总结短标题，并支持后续手动修改。

约束：TBox 本地优先；标题生成不得阻塞 Agent 回合；失败可回退截断标题；复用 active LLM **配置**（同一 profile），但 MUST NOT 复用对话 Agent 的系统提示词 / Skill / 工具 harness / session 轨迹。

## Goals / Non-Goals

**Goals:**
- 新建会话首条用户消息：截断占位 → 后台 Title Summarizer → 更新 title（侧栏 + 头栏同步）
- 用户可重命名；手动改过后 MUST NOT 被迟到的总结覆盖
- 聊天主区顶栏：左标题（可编辑）、右轨迹入口

**Non-Goals:**
- 不为标题单独选模型 / 单独计费配置
- 不把标题总结做成带工具的完整 Agent 循环
- 不批量回填历史会话标题
- 不把标题生成写入 agent session_log 轨迹

## Decisions

### 1. 占位 + 异步总结，不阻塞发送

- **选择**：`append_user_message` 仍同步写截断标题并返回；成功后由 Rust 侧后台任务调用 Title Summarizer 并 `UPDATE` title。
- **替代**：同步等 LLM 再返回 → 首条消息延迟明显，否决。
- **触发点**：仅在「新建会话」（`conversation_id` 原为 `None`）的第一次用户消息后触发一次。

### 2. 防覆盖：`title_locked` 标志

- **选择**：`conversations` 表增加 `title_locked INTEGER NOT NULL DEFAULT 0`（migration）。用户 `rename_conversation` 时置 `1`；异步总结写库前若 `title_locked=1` 则跳过。
- **替代**：仅比较「当前 title 是否仍等于截断占位」→ 用户若恰好改成相同字符串会误覆盖；标志更稳。

### 3. Title Summarizer = 独立单次 completion（不是 Agent）

- **选择**：新建轻量模块（如 `src-tauri/src/agent/title_summarizer.rs`）：
  - 自有极短 system + user（例如：「用中文生成 ≤20 字会话标题，只输出标题，无引号无解释」+ 用户首条消息，输入可截断到 ~500 字）
  - 解析当前 active LLM 配置，构造**独立**的 `Model`/`complete` 调用（独立消息列表）
  - 输出清洗：取首行、trim、去 markdown 围栏/引号、硬上限 40 字
  - Mock（`TBOX_AGENT_MOCK=1`）：确定性假标题，便于测试
- **MUST NOT**：拼装对话系统提示词、注入 Skill、注册/调用工具、走 agent loop / harness / session_log
- **可复用**：同一 active profile 的 endpoint / 模型 id / API key 解析逻辑
- **失败**：保留截断标题；默认静默，不阻断聊天

### 4. 前端同步

- **选择**：总结完成后 emit Tauri 事件 `conversation:title` `{ id, title }`，前端 store 更新 `items` 与头栏；另提供 `rename_conversation(id, title)` invoke。
- **头栏**：`HomePage` 顶部 `chat-chrome`：左可点击进入 input 编辑，失焦或 Enter 提交 rename；右「轨迹」链到现有 agent-runs 详情。草稿显示「新对话」，轨迹入口禁用。

### 5. 与主 Agent 回合的资源争用

- **选择**：Summarizer 使用独立消息列表与短超时；不插入主 Agent 消息流。本地小模型若单飞，可排队或超时失败回退截断标题。
- **Trade-off**：弱机上标题可能晚到或失败。

## Risks / Trade-offs

- [本地模型串行] → 短超时 + 失败保留截断；不阻塞聊天
- [LLM 胡写长文/多行] → 强约束解析：首行、硬截断、过滤围栏
- [用户秒改标题 vs 迟到总结] → `title_locked`
- [费用/配额] → 每会话最多一次自动总结

## Migration Plan

1. SQLite：`ALTER TABLE conversations ADD COLUMN title_locked INTEGER NOT NULL DEFAULT 0`（懒迁移）
2. 旧会话 `title_locked=0`；仅新会话触发 Summarizer
3. 回滚：忽略新列与 rename command；标题仍可读

## Open Questions

- （无阻塞项）头栏是否显示「生成中…」：默认先显示截断标题，Summarizer 返回后替换。
