## ADDED Requirements

### Requirement: Attribution Fields On Golden Cases
离线评测集（`cases.json`）中的 case MAY 包含可选字段 `expect_failure_class`（取值为 harness 定义的首错类别枚举之一）。逻辑层 MUST 在适用时校验实际归类与期望一致；未设置该字段的 case 行为与既有四字段契约兼容。

#### Scenario: Optional failure class on negative case
- **WHEN** 某 negative case 声明 `expect_failure_class` 为 `intent` 且 `expect_tool` 为空
- **THEN** 逻辑层在模型输出含工具调用时将该偏离归为 `intent` 并按期望断言

### Requirement: E2E Metrics Include Attribution
端到端层在既有四项成功率指标之外，MUST 输出首错类别计数汇总；无失败时各类计数可为 0。纯逻辑层测试 MUST 覆盖归因分类函数的确定性用例。

#### Scenario: Attribution printed with e2e metrics
- **WHEN** 执行带 `agent-eval` 的端到端评测且有已安装模型
- **THEN** 控制台报告同时包含四项比率与按类别的失败计数

## MODIFIED Requirements

### Requirement: Eval Suite Tiers Preserved
评测 MUST 保留两层入口：

- 纯逻辑层：解析/校验/容错/归因，无网络无权重，随 `cargo test` 必跑。
- 端到端层：`--features agent-eval` 加载已安装本地模型跑完整循环；无权重时跳过并明确提示需要先下载模型。

端到端层 MUST 输出至少 4 项指标：意图命中率、工具选择正确率、参数合法率、端到端成功率，以及首错类别计数。

#### Scenario: Logic-layer runs without weights
- **WHEN** `cargo test` 在无 GGUF 权重的 CI 环境运行
- **THEN** 纯逻辑层用例全部运行并断言通过，不依赖权重

#### Scenario: End-to-end prints metrics with installed model
- **WHEN** 本地已安装目录内 GGUF 模型且执行 `cargo test --lib --features agent-eval agent::harness::eval::e2e`
- **THEN** 输出意图命中率、工具选择正确率、参数合法率、端到端成功率四行报告，以及首错类别计数

#### Scenario: End-to-end skipped without weights
- **WHEN** 本地无已安装 GGUF 模型
- **THEN** 端到端测试打印「需要先下载模型」提示并跳过，不报错失败
