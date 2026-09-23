## MODIFIED Requirements

### Requirement: Conversation History
系统 SHALL 支持新建对话、按时间列出历史、打开已有会话、删除会话与**重命名会话标题**。空会话在发出第一条用户消息之前 MUST NOT 写入持久化存储。第一条用户消息 MUST 被持久化，并先写入本地短标题占位（由该消息**第一句话核心**截断得到，去掉请/帮我等套话）；随后系统 MUST 通过**独立的 Title Summarizer**（单次 LLM completion、自有短 prompt，仅基于第一句核心）异步生成更短的总结标题并更新该会话。Title Summarizer MUST 使用当前已配置 LLM 的连接配置，MUST NOT 复用对话 Agent 的系统提示词、Skill 注入、工具目录或 agent 工具循环。失败时 MUST 保留占位标题，MUST NOT 失败整次发送。若用户已手动重命名，迟到的自动总结 MUST NOT 覆盖用户标题。

#### Scenario: First message persists conversation
- **WHEN** 用户在空对话中发送第一条消息
- **THEN** 该会话与该条消息被写入本地 SQLite，历史列表出现该会话且标题先为该消息的截断占位

#### Scenario: LLM summarizes title asynchronously
- **WHEN** 新建会话的第一条用户消息已持久化且当前 LLM 可用
- **THEN** 系统在不阻塞 Agent 回合的前提下由 Title Summarizer 异步生成短标题并更新该会话；侧栏与聊天头栏展示更新后的标题

#### Scenario: Title from first-sentence core
- **WHEN** 系统为某会话生成自动标题或截断占位
- **THEN** 输入以用户首条消息的第一句话为核心（去掉请/帮我等套话），MUST NOT 把后续句子拼进标题；总结标题应为该句意图的短名词短语

#### Scenario: Short first sentence skips LLM title
- **WHEN** 用户首条消息经第一句核心提取后长度不超过 16 个字
- **THEN** 系统直接将该短句作为会话标题，MUST NOT 再发起 Title Summarizer 的 LLM completion

#### Scenario: Title summarizer is isolated from chat agent prompt
- **WHEN** 系统为某会话生成自动标题（且第一句核心超过短句阈值、需要 LLM）
- **THEN** 发往模型的请求仅含 Summarizer 自有短提示与用户首条消息摘要，不含对话系统提示词、工具 schema 或 Skill 正文

#### Scenario: Title summarize failure keeps placeholder
- **WHEN** 标题总结调用失败或超时
- **THEN** 会话标题保持截断占位，聊天与 Agent 回合仍可继续

#### Scenario: Manual rename locks title
- **WHEN** 用户将某会话标题改为自定义文案
- **THEN** 该标题持久化；之后若仍有未完成的自动总结完成，MUST NOT 覆盖用户标题

#### Scenario: Empty new chat not listed
- **WHEN** 用户点击新建对话但尚未发送任何消息
- **THEN** 历史列表不出现新的空白会话记录

#### Scenario: Delete conversation
- **WHEN** 用户删除一条已持久化的会话
- **THEN** 该会话及其消息从存储中移除，且不再出现在历史列表

## ADDED Requirements

### Requirement: Chat Window Chrome Header
聊天主区域顶部 SHALL 提供一条状态栏（chrome）：左侧展示当前会话标题（草稿或空会话可用占位文案如「新对话」），右侧提供进入本会话完整交互轨迹的入口。标题在已持久化会话上 MUST 可就地编辑并保存。轨迹入口 MUST NOT 再以消息列表上的绝对定位浮层作为唯一入口。

#### Scenario: Header shows title and trajectory
- **WHEN** 用户打开一条已持久化会话
- **THEN** 聊天区顶部状态栏左侧显示该会话标题，右侧显示轨迹入口

#### Scenario: Edit title from header
- **WHEN** 用户在状态栏编辑标题并确认
- **THEN** 新标题写入存储，侧栏历史列表同步为新标题

#### Scenario: Draft header without trajectory
- **WHEN** 用户处于尚未持久化的空草稿对话
- **THEN** 状态栏显示占位标题；轨迹入口禁用或不可进入有效轨迹页
