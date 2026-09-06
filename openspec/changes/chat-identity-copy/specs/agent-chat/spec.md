## ADDED Requirements

### Requirement: User Message Avatar
每条用户消息 SHALL 在气泡旁展示用户头像。头像 MUST 来自设置中选定的预设（见 `settings-general`）；同一应用内所有会话 MUST 使用当前选定的用户头像。

#### Scenario: User message shows avatar
- **WHEN** 对话中存在一条用户消息
- **THEN** 该消息旁显示当前用户头像预设，而非仅有气泡文本

#### Scenario: Avatar reflects settings change
- **WHEN** 用户在设置中更换用户头像预设后返回对话
- **THEN** 已有与新发用户消息均展示新选定的头像

### Requirement: Assistant Display Name
助手正文消息行 SHALL 展示助手显示名（与会话随机头像并存）。显示名 MUST 来自设置中的全局偏好；未配置时 MUST 使用产品默认名（如「TBox」）。助手头像仍遵循既有「每会话随机、会话内稳定」规则，MUST NOT 因本需求改为可在设置中固定。

#### Scenario: Assistant name visible on reply
- **WHEN** 助手回合包含正文气泡
- **THEN** 气泡旁/上方可见当前助手显示名，并与该会话的助手头像一同展示

#### Scenario: Renamed assistant appears in chat
- **WHEN** 用户在设置中修改助手显示名后返回对话
- **THEN** 后续与历史助手正文行均展示新名称（无需重建会话）

## MODIFIED Requirements

### Requirement: Message Copy
系统 SHALL 为以下内容提供统一的复制操作：用户消息正文、助手消息正文、已展开工具步骤的入参、已展开工具步骤的出参（结果文本）。鼠标悬停（或聚焦）目标区域时 MUST 出现样式一致的图标复制按钮；点击后将对应**纯文本**复制到系统剪贴板（助手正文为未渲染的 Markdown 源文本；工具入参为可读序列化文本，如格式化 JSON；工具出参为结果字符串），并在约 1–2 秒内给出勾选成功反馈。思考过程步骤 MUST NOT 因本需求强制提供复制。

#### Scenario: Copy assistant message
- **WHEN** 用户 hover 一条助手正文并点击复制按钮
- **THEN** 该正文的原始 Markdown 源文本被复制到剪贴板，按钮给出复制成功反馈

#### Scenario: Copy user message
- **WHEN** 用户 hover 一条自己发出的消息并点击复制按钮
- **THEN** 该消息文本被复制到剪贴板，按钮给出复制成功反馈

#### Scenario: Copy tool args and result
- **WHEN** 用户展开某工具步骤并分别点击入参、出参的复制
- **THEN** 剪贴板依次为入参的可读文本与出参文本；两者可独立复制

#### Scenario: No copy required on reasoning
- **WHEN** 用户仅展开思考步骤
- **THEN** 界面不必提供思考正文的复制按钮（允许省略）

### Requirement: Agent Avatar Variety
助手消息头像 SHALL 提供一组活泼的动物/趣味图标与渐变配色，而非单一机器人图标。每个会话创建时 SHALL 随机确定一个头像，同一会话内（含重开后）保持不变。用户头像 MUST 由全局预设偏好决定（见 User Message Avatar / settings-general），MUST NOT 随会话随机变化，亦 MUST NOT 被助手头像随机逻辑覆盖。

#### Scenario: New conversation gets random avatar
- **WHEN** 用户创建新会话并发送消息
- **THEN** 助手消息使用从头像集合中随机选取的图标+配色，与之前会话可能不同

#### Scenario: Avatar stable within conversation
- **WHEN** 用户在同一会话中收发多条消息或重开该会话
- **THEN** 助手头像保持一致

#### Scenario: User avatar independent of conversation
- **WHEN** 用户在多个会话间切换
- **THEN** 用户消息头像保持为当前全局预设，不随会话切换而随机更换
