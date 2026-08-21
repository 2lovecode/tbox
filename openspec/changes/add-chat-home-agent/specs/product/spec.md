## MODIFIED Requirements

### Requirement: Local-first Toolbox
系统 SHALL 作为本地桌面应用运行，核心工具在无网络时仍可用（明确依赖外网的工具除外）。在用户已下载并启用本地模型的前提下，对话 Agent 的本地提供者路径 MUST 同样可在无外网时使用。

#### Scenario: Offline encoding tool
- **WHEN** 用户在断网环境下打开 Base64 / 哈希等本地计算类工具
- **THEN** 工具可正常完成计算且不依赖云端服务

#### Scenario: Offline agent after local model install
- **WHEN** 用户已下载并启用本地模型且处于断网环境
- **THEN** 可通过对话 Agent 调用已注册的纯计算工具并得到结果，且该路径不依赖云端 LLM

---

### Requirement: Target Users
系统 SHALL 优先服务后端工程师、测试工程师与运维工程师的日常开发与排查场景。工具发现入口 MUST 为工具箱（及 Spotlight），MUST NOT 再依赖根路由上的工具卡片网格。

#### Scenario: Tool discoverability for backend workflows
- **WHEN** 用户打开工具箱
- **THEN** 可发现编码转换、JSON/加密、时间、网络、数据库等后端常用类别
