## ADDED Requirements

### Requirement: Compact Default Window
应用默认主窗口尺寸 SHALL 紧凑于以往过大默认值：宽度约 1200px、高度约 780px（允许小幅偏差），且 MUST 仍保持侧栏 + 主内容双栏布局（不得因默认宽度落入窄屏断点而单列化）。用户手动调整后的窗口尺寸行为遵循 OS / Tauri 既有规则，本要求仅约束首次默认尺寸。

#### Scenario: Fresh launch default size
- **WHEN** 用户在无持久化异常窗口状态的情况下启动应用
- **THEN** 主窗口以约 1200×780 量级打开，并同时可见侧栏与对话主区（双栏），而非仅侧栏铺满或约 1400×1050

### Requirement: Chat Layout Without Identity Chrome
聊天界面 MUST NOT 展示用户头像、助手头像或助手显示名。用户消息 SHALL 以弱强调气泡展示（淡底或细边，避免高饱和强渐变）；助手正文 SHALL 不以强对比气泡强调（纯文本块即可）。

#### Scenario: No avatars or assistant name
- **WHEN** 用户查看含用户与助手消息的对话
- **THEN** 消息旁无头像，助手正文旁无显示名

#### Scenario: Subtle user bubble only
- **WHEN** 对话中同时存在用户消息与助手正文
- **THEN** 用户消息呈弱强调气泡，助手正文无强烈气泡底色

## MODIFIED Requirements

### Requirement: Message Copy
系统 SHALL 为以下内容提供复制操作：用户消息正文、助手消息正文、已展开工具步骤的入参、已展开工具步骤的出参（结果文本）。用户与助手正文的复制控件 MUST 在鼠标悬停（或聚焦）对应消息时出现在该消息**下方**，样式为图标按钮；点击后将对应纯文本复制到剪贴板（助手正文为未渲染 Markdown 源文本；工具入参为可读序列化文本；工具出参为结果字符串），并在约 1–2 秒内给出勾选成功反馈。思考过程步骤 MUST NOT 因本需求强制提供复制。

#### Scenario: Copy assistant message
- **WHEN** 用户 hover 一条助手正文并点击其下方复制按钮
- **THEN** 该正文的原始 Markdown 源文本被复制到剪贴板，按钮给出复制成功反馈

#### Scenario: Copy user message
- **WHEN** 用户 hover 一条自己发出的消息并点击其下方复制按钮
- **THEN** 该消息文本被复制到剪贴板，按钮给出复制成功反馈

#### Scenario: Copy tool args and result
- **WHEN** 用户展开某工具步骤并分别点击入参、出参的复制
- **THEN** 剪贴板依次为入参的可读文本与出参文本；两者可独立复制

#### Scenario: No copy required on reasoning
- **WHEN** 用户仅展开思考步骤
- **THEN** 界面不必提供思考正文的复制按钮（允许省略）

## REMOVED Requirements

### Requirement: Agent Avatar Variety
**Reason**: 对话 UI 不再展示助手头像，会话随机头像策略废止  
**Migration**: 删除聊天中的助手头像渲染；相关前端常量可删除
