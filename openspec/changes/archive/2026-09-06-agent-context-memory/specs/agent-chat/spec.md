## ADDED Requirements

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
