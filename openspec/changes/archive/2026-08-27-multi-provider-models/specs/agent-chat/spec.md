## ADDED Requirements

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
