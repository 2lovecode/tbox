## MODIFIED Requirements

### Requirement: Offline Evaluation Suite
系统 SHALL 内置离线评测集（golden cases：用户输入、预期工具与参数要点、预期输出要点），覆盖全部已注册工具类别、中英文表达、以及无需工具的闲聊负例。评测 MUST 提供两层入口：纯逻辑层（解析/校验/容错，无网络无权重，随 `cargo test` 必跑）与端到端层（加载已安装本地模型跑完整循环，权重缺失时跳过并提示）。端到端层 MUST 输出至少四项指标：意图命中率、工具选择正确率、参数合法率、端到端成功率。评测集条目数 MUST ≥ 60 条；每个注册工具 MUST 至少被 2 条 case 命中（1 条中文问法 + 1 条英文/多样化问法为最低门槛）；评测集 MUST 包含至少 6 条多步调用 case 与至少 6 条无可用工具负例 case。

#### Scenario: Logic-layer eval runs in CI
- **WHEN** 执行 `cargo test`
- **THEN** 评测集纯逻辑层用例全部运行并对解析与校验行为断言；条目数 ≥ 60 且全部 16 工具覆盖

#### Scenario: End-to-end report with model installed
- **WHEN** 本地已安装目录内 GGUF 模型且运行端到端评测
- **THEN** 输出意图命中率、工具选择正确率、参数合法率、端到端成功率报告

#### Scenario: Skipped gracefully without weights
- **WHEN** 本地无已安装模型时运行端到端评测
- **THEN** 评测跳过并明确提示需要先下载模型，不报错失败

#### Scenario: Multi-step case covered
- **WHEN** 用户请求需要 ≥2 个工具串接（如「先 base64 解码再算哈希」）
- **THEN** 评测集中至少有 6 条 multi-step case 可经单测断言其预期工具链

#### Scenario: Negative case covered
- **WHEN** 用户请求属于「无可用工具」范畴（如天气、闲聊、计算器）
- **THEN** 评测集中至少有 6 条 negative case 期望无 Tool 调用