## ADDED Requirements

### Requirement: Eval Suite Minimum Size
离线评测集（`src-tauri/src/agent/harness/eval/cases.json`）MUST 至少包含 60 条 golden cases。覆盖：

- 16 个注册工具（12 既有 + 4 新增）每个至少 2 条（中英文各 1 条为最低门槛）。
- 至少 6 条「多步调用」case（一次对话中 ≥2 个工具调用）。
- 至少 6 条「无可用工具」负例（聊天、天气、计算器等需要调用模型自身或第三方服务的请求）。
- 每个 case id MUST 唯一；每条 case MUST 含 `user_input` / `expect_tool`（可空）/ `expect_args` / `model_output` 四字段。

#### Scenario: Suite meets size floor
- **WHEN** 加载 `cases.json` 并执行 `cases_wellformed_and_sized` 测试
- **THEN** 断言总条目数 ≥ 60、id 唯一、全部 16 工具至少被 2 个 case 命中

#### Scenario: New app-layer tools covered
- **WHEN** eval 集执行 `logic_layer_parse_matches_expectations`
- **THEN** 4 个新工具（`json.to_query` / `json.flatten` / `url.parse` / `form.parse`）各自至少有 1 条 case 在 `expect_tool` 字段命中

### Requirement: Eval Suite Tiers Preserved
评测 MUST 保留两层入口：

- 纯逻辑层：解析/校验/容错，无网络无权重，随 `cargo test` 必跑。
- 端到端层：`--features agent-eval` 加载已安装本地模型跑完整循环；无权重时跳过并明确提示需要先下载模型。

端到端层 MUST 输出至少 4 项指标：意图命中率、工具选择正确率、参数合法率、端到端成功率。

#### Scenario: Logic-layer runs without weights
- **WHEN** `cargo test` 在无 GGUF 权重的 CI 环境运行
- **THEN** 纯逻辑层用例全部运行并断言通过，不依赖权重

#### Scenario: End-to-end prints metrics with installed model
- **WHEN** 本地已安装目录内 GGUF 模型且执行 `cargo test --lib --features agent-eval agent::harness::eval::e2e`
- **THEN** 输出意图命中率、工具选择正确率、参数合法率、端到端成功率四行报告

#### Scenario: End-to-end skipped without weights
- **WHEN** 本地无已安装 GGUF 模型
- **THEN** 端到端测试打印「需要先下载模型」提示并跳过，不报错失败