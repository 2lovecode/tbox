## ADDED Requirements

### Requirement: Multiple Provider Profiles
系统 SHALL 支持同时保存多个提供方配置（profile）。每个 profile MUST 含唯一 id、显示名、provider 预设 id、协议、base_url、model；云端 profile 的 API Key MUST 沿用现有 AES-256-GCM 方案加密存储，且每个 profile 的 Key 独立保存。非敏感字段 MUST 以本地 JSON 持久化。

#### Scenario: Save two cloud profiles
- **WHEN** 用户先后保存 DeepSeek 与 Anthropic 两个 profile（各含 API Key）
- **THEN** 两个 profile 同时出现在配置列表中，各自的 API Key 独立加密存储，互不影响

#### Scenario: Profile CRUD
- **WHEN** 用户新建、编辑或删除一个 profile
- **THEN** 列表立即反映变更；删除 profile 的同时清除其加密密钥；系统 MUST NOT 允许删除后留下悬空的当前激活指向

### Requirement: Active Profile
系统 SHALL 维护一个"当前激活 profile"。Agent 循环、连通性测试等一切推理相关行为 MUST 且仅 MUST 使用激活 profile 的配置。同一时刻 MUST 只有唯一激活 profile。

#### Scenario: Switch active profile
- **WHEN** 用户把激活 profile 从 A 切换为 B
- **THEN** 后续推理请求全部使用 B 的 provider/协议/base_url/model/API Key，不再使用 A

### Requirement: Legacy Config Migration
系统 SHALL 在首次读取时把旧版单配置（`llm_config.json` + `llm_secret.bin`）自动迁移为一条 profile 并设为激活，MUST NOT 丢失或重复解密用户的 API Key。

#### Scenario: Migrate existing single config
- **WHEN** 用户带着旧版单配置启动新版本
- **THEN** 迁移生成的 profile 成为唯一且激活的配置，其 API Key 仍可用于推理；旧单配置读取路径不再被前端使用

#### Scenario: Fresh install unaffected
- **WHEN** 用户全新安装、无任何旧配置
- **THEN** profile 列表为空（或仅默认 local profile），激活状态为空，界面引导用户创建第一个配置
