## ADDED Requirements

### Requirement: Generation Parameters On Profiles
每个 LLM profile SHALL 可持久化可选生成参数：`temperature`（浮点）、`top_p`（浮点）、`max_tokens`（正整数）；当 `provider` 为 `local` 时另可持久化 `n_ctx`（正整数，嵌入式上下文长度）。上述字段均为可选；缺省时 Agent / 连通性测试 MUST 使用后端内置默认。保存与列表 API MUST 往返这些字段；非法值（越界、非数字）MUST 在保存时拒绝并返回可读错误，不得写入磁盘。

#### Scenario: Save temperature and top_p on cloud profile
- **WHEN** 用户为 DeepSeek profile 设置 temperature=0.3、top_p=0.9、max_tokens=2048 并保存
- **THEN** 再次列出 profiles 时该三条数值与保存时一致

#### Scenario: Local profile persists n_ctx
- **WHEN** 用户为 local profile 设置 n_ctx=8192 并保存
- **THEN** 磁盘上的 profile 含 n_ctx=8192，且后续 embedded 加载使用该上下文长度

#### Scenario: Invalid temperature rejected
- **WHEN** 用户保存 temperature 超出允许范围（例如负数或显著大于实现上限）
- **THEN** 保存失败并提示参数非法，原 profile 不被破坏性覆盖

### Requirement: Settings UI For Generation Parameters
设置页编辑 profile 时 SHALL 提供生成参数表单（temperature、top_p、max_tokens；local 时显示 n_ctx），并展示各参数的默认值提示。用户清空某字段后保存，系统 MUST 将该字段视为未设置并回退默认。

#### Scenario: Clear field reverts to default
- **WHEN** 用户清空已保存的 temperature 并保存 profile
- **THEN** 该 profile 不再携带自定义 temperature，推理使用内置默认

### Requirement: Agent Loop Honors Active Profile Generation Params
Agent 补全（云端 / Ollama / embedded）MUST 使用当前激活 profile 的生成参数（若已设置）构造请求或采样链；MUST NOT 静默忽略已保存的自定义参数。

#### Scenario: Cloud completion uses max_tokens from profile
- **WHEN** 激活云端 profile 的 max_tokens=1024 且发起 Agent 对话
- **THEN** 发往该后端的补全请求携带对应的最大输出长度限制（或等价字段）

## MODIFIED Requirements

### Requirement: Multiple Provider Profiles
系统 SHALL 支持同时保存多个提供方配置（profile）。每个 profile MUST 含唯一 id、显示名、provider 预设 id、协议、base_url、model，以及可选生成参数（见 Generation Parameters On Profiles）；云端 profile 的 API Key MUST 沿用现有 AES-256-GCM 方案加密存储，且每个 profile 的 Key 独立保存。非敏感字段（含生成参数）MUST 以本地 JSON 持久化。

#### Scenario: Save two cloud profiles
- **WHEN** 用户先后保存 DeepSeek 与 Anthropic 两个 profile（各含 API Key）
- **THEN** 两个 profile 同时出现在配置列表中，各自的 API Key 独立加密存储，互不影响

#### Scenario: Profile CRUD
- **WHEN** 用户新建、编辑或删除一个 profile
- **THEN** 列表立即反映变更；删除 profile 的同时清除其加密密钥；系统 MUST NOT 允许删除后留下悬空的当前激活指向
