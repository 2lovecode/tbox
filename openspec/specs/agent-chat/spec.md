# Agent Chat

## Purpose

对话首页、会话历史与 Rust 侧 Agent 工具循环（预置 Skill、白名单纯计算工具、流式事件与 LLM 不可用引导）。
## Requirements
### Requirement: Conversation Homepage
系统 SHALL 将应用根路由 `/` 渲染为当前对话，而不是工具卡片网格。

#### Scenario: Launch opens chat
- **WHEN** 用户启动应用并进入根路由
- **THEN** 主区域展示对话界面（空态欢迎或上次会话），而不是工具卡片列表

#### Scenario: Return from tool page
- **WHEN** 用户从某一工具页返回根路由
- **THEN** 回到离开前的那条会话（若尚无会话则展示空态），而不是工具网格

### Requirement: Conversation History
系统 SHALL 支持新建对话、按时间列出历史、打开已有会话、删除会话。空会话在发出第一条用户消息之前 MUST NOT 写入持久化存储。第一条用户消息 MUST 被持久化，并用于生成短标题。

#### Scenario: First message persists conversation
- **WHEN** 用户在空对话中发送第一条消息
- **THEN** 该会话与该条消息被写入本地 SQLite，历史列表出现该会话且标题来自该消息的截断

#### Scenario: Empty new chat not listed
- **WHEN** 用户点击新建对话但尚未发送任何消息
- **THEN** 历史列表不出现新的空白会话记录

#### Scenario: Delete conversation
- **WHEN** 用户删除一条已持久化的会话
- **THEN** 该会话及其消息从存储中移除，且不再出现在历史列表

### Requirement: Agent Tool Loop
系统 SHALL 在 Rust 侧执行 Agent 循环：系统提示构建、输出解析、参数校验与修复重试由统一 harness 策略层承担（见 agent-tool-harness 能力）；策略按当前后端/模型插拔，所有后端 MUST 经由该层。循环将系统提示、检索到的 Skill、会话消息发给当前 LLM；若模型返回 tool call，则仅调度注册表中的工具并把结果回填；参数校验不合格或调用被拒绝时 MUST 在修复预算内回填错误并重试（reask），而不是直接终止回合。前端 MUST 以流式事件按时间线展示 token、reasoning 与工具调用（名称、参数、结果或失败），且工具步骤 MUST 出现在同一助手回合轨迹内（不得仅在回合进行中短暂显示、结束后从历史中消失）。

#### Scenario: Single tool call then answer
- **WHEN** 用户发送需要 Base64 解码的请求且本地或云端 LLM 可用
- **THEN** 对话中出现对应工具调用记录与结果，并跟有助手对结果的说明

#### Scenario: Sequential tool calls
- **WHEN** 用户请求先 Base64 解码再计算哈希，且两个工具均已注册
- **THEN** 系统按循环依次执行这两个工具，并在同一轮对话轨迹中按序展示两次调用与最终回复

#### Scenario: Invalid tool call repaired via reask
- **WHEN** 模型返回的工具调用参数不符合 JSON Schema
- **THEN** 系统不调用底层 command，将校验错误回填给模型并重新请求；修复预算内得到合法调用则继续执行，预算耗尽则按错误语义收尾

#### Scenario: Cancel in-flight turn
- **WHEN** 用户在 Agent 循环尚未完成时取消发送
- **THEN** 循环停止；已持久化的消息与已收到的轨迹步骤保留；进行中的助手输出标记为中断而非删除整条会话

#### Scenario: Tool calls visible after turn completes
- **WHEN** 含工具调用的助手回合结束并写入存储
- **THEN** 刷新或重开该会话后，工具步骤仍出现在该回合时间线中

### Requirement: Allowlisted Pure-compute Tools
系统 SHALL 仅允许 Agent 调用注册表中的工具。本 change 注册的工具 MUST 全部为 `side_effect=none` 的纯计算：JSON（id 9）、Base64（10）、哈希（11）、JWT 解析（14，不含验签/加解密默认路径）、时间戳（16）、编码（19）、XML（20）、YAML（21）、UUID（30）、Cron（31）、数字（32）、字符集（33）。未注册工具 MUST 被拒绝且不得执行。

#### Scenario: Unregistered tool rejected
- **WHEN** 模型请求调用未在注册表中的工具 id（例如 HTTP 或数据库）
- **THEN** 系统不执行该工具，并将拒绝原因作为工具错误回填给模型

#### Scenario: Invalid arguments rejected
- **WHEN** 模型对已注册工具给出不符合 JSON Schema 的参数
- **THEN** 系统不调用底层 command，并将校验错误回填给模型

#### Scenario: Tool failure explained
- **WHEN** 已注册工具执行失败（例如非法 Base64）
- **THEN** 对话展示该次工具失败，模型收到错误文本并生成说明，应用不崩溃

### Requirement: Preset Skills Only
系统 SHALL 为每个本 change 注册的工具提供预置 Skill 说明书，并按用户问题检索后只注入少量相关 Skill。Skill MUST NOT 注册新的可执行器。用户安装第三方 Skill 不在本 change 范围。

#### Scenario: Relevant skill injected
- **WHEN** 用户询问与 JWT 解析相关的问题
- **THEN** Agent 上下文包含 JWT 相关预置 Skill，且不把全部预置 Skill 一并注入

#### Scenario: Skills cannot add tools
- **WHEN** 系统加载预置 Skill
- **THEN** 可调用工具集合仍仅来自注册表，不会因 Skill 文本增加新的执行入口

### Requirement: LLM Unavailable Guidance
当当前提供者不可用（未下载本地模型、sidecar 未运行、Ollama 不可达或未 pull 模型、云端未配置、所选协议/端点请求失败、或所选为暂不支持的 OAuth 预设）时，系统 SHALL 在对话界面给出可执行的明确提示（前往设置下载、pull、切换已配置提供商或更换协议），MUST NOT 在无提示的情况下静默失败。

#### Scenario: No local model and no cloud
- **WHEN** 用户在未下载本地模型、未配置可用 Ollama 模型、且未配置可用云端 LLM 时尝试发送消息
- **THEN** 界面提示需要下载模型、配置 Ollama 或配置 LLM，并指向设置，不发起无目标的推理请求

#### Scenario: Unsupported OAuth provider selected
- **WHEN** 当前保存的提供商为需要 OAuth 且本应用暂不支持的预设
- **THEN** 界面提示该提供商暂不支持，并引导用户改选其它提供商，不发起推理请求

### Requirement: Protocol-Aware LLM Backend
系统 SHALL 按当前保存的提供商与协议，经 Rust 侧 LLM 客户端（优先使用现成多协议库）发起 Agent 补全与工具调用。在配置完整时，`anthropic_messages`、`openai_responses`、`gemini_native`、`ollama_native` 与 `openai_chat` MUST 均可作为有效后端，不得仅因非 OpenAI Chat Completions 而一律判定不可用。配置不完整、OAuth 未支持、或本地/Ollama 模型缺失时 MUST 返回明确不可用，MUST NOT 静默改打其它云端。

#### Scenario: Anthropic messages succeeds when configured
- **WHEN** 当前配置为 Anthropic（或兼容）端点、协议为 `anthropic_messages`、且 API Key 与模型已配置
- **THEN** Agent 循环可向该端点发起请求，不再因协议非 OpenAI 而恒为不可用

#### Scenario: Ollama native when selected
- **WHEN** 当前提供者为 `ollama`、协议为 `ollama_native`、且选定模型在本机可用
- **THEN** Agent 循环经 Ollama 完成至少一轮补全（含工具循环所需的模型回合）

#### Scenario: No silent cloud fallback
- **WHEN** 当前为 `local` 且无已启用模型，或为 OAuth 预设，或为缺少密钥的云端配置
- **THEN** Agent 不发起隐式云端请求，并向用户返回与 LLM Unavailable Guidance 一致的提示

---


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
消息区 UI SHALL 统一视觉规范：用户/助手气泡区分对齐与配色，助手消息支持流式输出指示（输出中状态），工具调用步骤 MUST 展示工具名、状态（运行中/完成/失败）、参数与结果（等宽字体、可滚动、限高折叠），并嵌入助手回合时间线而非与消息流脱节的孤立区域；错误提示条 MUST 与消息流视觉区分且不遮挡输入区。

#### Scenario: Streaming assistant indicator
- **WHEN** 助手回复正在流式输出
- **THEN** 消息区呈现输出中状态指示，结束后指示消失

#### Scenario: Tool call card presentation
- **WHEN** 一轮对话包含工具调用
- **THEN** 工具步骤按顺序展示工具名、状态徽标、参数与结果，参数/结果以等宽字体限高展示并可滚动

#### Scenario: Error bar stays visible
- **WHEN** 本轮出现错误提示
- **THEN** 错误条在输入区上方可见，不随消息滚动隐藏，不遮挡输入框

### Requirement: In-chat Model Switcher
聊天界面 SHALL 在**输入框区域**（主流聊天产品样式）提供模型切换器：以「提供方分组 + 该提供方可选模型」的粒度实时切换，而不只是切换提供方。切换器 MUST 展示每个已保存 profile 下的可用模型列表（云端经 `/models` 拉取、Ollama 经 `/api/tags` 拉取、本地列出已安装 GGUF），点击某模型即把该模型写入所属 profile 并设为激活，下一轮消息生效。无法列出模型列表的协议（如 anthropic_messages）MUST 提供手动输入模型的入口。切换 MUST NOT 打断进行中的流式回合。

#### Scenario: Switch model mid-conversation
- **WHEN** 用户在某会话中通过输入框切换器把模型从 A 提供方的模型 M1 切到 M2
- **THEN** 激活 profile 的 model 字段更新为 M2，下一条用户消息发起的推理使用 M2；已展示的历史消息不变

#### Scenario: Models fetched per provider
- **WHEN** 用户打开切换器且某云端/Ollama profile 可达
- **THEN** 该 profile 名下展示从端点拉取的模型列表；当前使用的模型带选中标识

#### Scenario: In-flight turn not interrupted
- **WHEN** 用户在流式输出进行中切换模型
- **THEN** 当前回合继续用原模型完成或被用户显式取消，系统 MUST NOT 中途换模型拼接输出

#### Scenario: No usable profile
- **WHEN** 用户没有任何已保存 profile（或激活 profile 配置不完整）时打开聊天
- **THEN** 切换器展示空态并引导前往设置，发送消息行为遵循既有 LLM Unavailable Guidance，不静默失败

### Requirement: Agent Avatar Variety
助手消息头像 SHALL 提供一组活泼的动物/趣味图标与渐变配色，而非单一机器人图标。每个会话创建时 SHALL 随机确定一个头像，同一会话内（含重开后）保持不变。用户头像样式不受影响。

#### Scenario: New conversation gets random avatar
- **WHEN** 用户创建新会话并发送消息
- **THEN** 助手消息使用从头像集合中随机选取的图标+配色，与之前会话可能不同

#### Scenario: Avatar stable within conversation
- **WHEN** 用户在同一会话中收发多条消息或重开该会话
- **THEN** 助手头像保持一致

### Requirement: Assistant Turn Trajectory Display
聊天界面 SHALL 将同一助手回合内的思考、工具调用与正文按发生顺序渲染为一条时间线（轨迹）。流式进行中各步骤 MUST 默认展开并随事件增量更新；回合结束后（及历史重开）思考与工具步骤 MUST 默认收起，正文 MUST 保持展开。用户可手动切换某步展开/收起；重开会话后 MUST 恢复上述默认折叠规则。

#### Scenario: Streaming turn shows ordered steps expanded
- **WHEN** 一轮包含思考与工具调用的回复正在流式生成
- **THEN** 界面按顺序展示思考、工具（参数与结果状态）、正文，且各步骤默认展开并随事件更新

#### Scenario: Completed turn collapses process steps
- **WHEN** 该回合完成或用户重新打开含轨迹的历史会话
- **THEN** 思考与工具步骤默认收起，助手正文保持展开可见

#### Scenario: Manual expand does not persist across reopen
- **WHEN** 用户在已完成回合中手动展开某思考或工具步骤后关闭并重新打开该会话
- **THEN** 该步骤再次以默认收起状态展示

### Requirement: Trajectory Persistence
带过程步骤的助手消息 SHALL 将有序轨迹持久化到本地 SQLite（`messages` 新增 `trajectory_json` 列，或等价结构化字段）；历史会话重开时 MUST 按原顺序恢复时间线。已存在的数据库 MUST 通过向后兼容 migration 加列；旧消息无轨迹时 MUST 由 `reasoning`、`tool_calls_json`、`content` 合成一条降级时间线（顺序：思考 → 工具 → 正文）。

#### Scenario: Reopen restores trajectory order
- **WHEN** 用户重新打开一条含多步工具与思考的历史会话
- **THEN** 时间线步骤顺序与生成时一致，工具调用可见而不丢失

#### Scenario: Legacy message without trajectory
- **WHEN** 旧库消息仅有 content/reasoning/tool_calls 而无轨迹字段
- **THEN** 界面仍展示合成时间线，应用不崩溃

### Requirement: Stream Mode Indication
Agent 循环在每轮模型调用开始时 SHALL 经 `agent-event` 下发流式模式元数据：`live`（真流式）或 `fallback`（整段完成后分块）。当模式为 `fallback` 时，前端 MUST 在该助手回合时间线上提供轻量可见提示（例如「整段生成」角标）；`live` 时 MUST NOT 强制显示该提示。

#### Scenario: Fallback mode shows badge
- **WHEN** 当前后端不支持真流式并以分块降级推送
- **THEN** 前端在本回合时间线显示 fallback 轻提示，且仍逐步渲染文本

#### Scenario: Live mode without fallback badge
- **WHEN** 后端以真流式推送 token/reasoning
- **THEN** 不显示「整段生成」类 fallback 提示

### Requirement: Context Usage Display
聊天界面 SHALL 展示当前会话发往模型的上下文使用情况：至少包含已用估算与窗口上限（或百分比）。数据 MUST 来自 Rust 侧与压缩触发同源的预算快照，MUST NOT 仅用前端消息字符数冒充。接近压缩阈值时 MUST 有轻量可见提示（文案实现自定）。

#### Scenario: Usage visible during long chat
- **WHEN** 多轮工具调用后上下文占用上升
- **THEN** 界面显示已用/上限（或百分比）且数值随回合更新

#### Scenario: Near-threshold hint
- **WHEN** 使用率达到或超过配置的压缩阈值（默认 80%）
- **THEN** 用户可见接近或正在压缩的提示，且不阻断发送

### Requirement: Session Compression Visibility
当系统对本会话 Runtime 上下文执行压缩时，聊天界面 SHALL 可感知（轻提示或轨迹元信息）。用户查看的历史消息与工具原文 MUST 仍可从 Audit 存储回看（或提供「查看原文」入口）；MUST NOT 因压缩而从 UI 删除已展示的助手回合时间线。

#### Scenario: Compress does not erase timeline
- **WHEN** 系统压缩了若干旧工具结果后用户滚动历史
- **THEN** 时间线仍在；被压缩项可区分或可打开原文，应用不崩溃

### Requirement: Retrieved Memory In Agent Loop
Agent 循环构建发往模型的上下文时，SHALL 在启用用户记忆的前提下注入检索到的相关记忆（见 `user-memory`），且注入占用 MUST 反映在预算快照的 `memory` 分项中。关闭自动记忆或无生效记忆时，循环行为与既有 Agent Tool Loop 一致。

#### Scenario: Loop includes memory when available
- **WHEN** 存在相关生效记忆且自动记忆未关闭，用户发送新消息
- **THEN** 该回合模型请求的上下文包含检索到的记忆片段，且预算快照含 memory 分项
