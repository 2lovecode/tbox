# App-Layer Tools Registered

## Purpose

把应用层高频任务（JSON→query、嵌套平铺、URL 拆解、表单解析）注册进 Agent 工具表，让内置小模型在 Agent 对话中一句话完成这些任务；`json.to_query` 复用既有前端工具的 Rust 实现，保持单一真相来源。

## Requirements

### Requirement: App-Layer Tools In Agent Registry
Agent 注册表 MUST 新增 4 个应用层工具，全部 `side_effect=none`：

- `json.to_query`：输入 `{"input": "<JSON 字符串>"}`，输出 URL-encoded query string（key/value 都用 percent-encoding）。内部复用既有 `commands::json::json_to_query_params` 的 `encoded` 字段，前端 `JsonToQuery.vue` 继续走原 command 不受影响。
- `json.flatten`：输入 `{"input": "<JSON 字符串>"}`，输出平铺后的 JSON 对象（嵌套对象路径用 `[key]`、数组下标用 `[i]`，与 `json.to_query` 同一路径风格；标量值序列化为字符串）。
- `url.parse`：输入 `{"input": "<URL 字符串>"}`，输出结构化 JSON（`scheme` / `host` / `port` 可选 / `path` 数组 / `query` 对象 / `fragment` 可选）。
- `form.parse`：输入 `{"input": "<application/x-www-form-urlencoded 字符串>"}`，输出 JSON 对象（重复键合并为数组，空字符串返回空对象）。

每个工具 MUST 拥有对应的 Skill 文档（应用场景 + 用户问法 → 工具调用样本），与注册表同源；eval 集 MUST 覆盖 ≥3 条边界用例。

#### Scenario: JSON to query via Agent
- **WHEN** 用户在 Agent 对话中请求「把 `{"aa":"bb"}` 转成 URL query string」且本地或云端 LLM 可用
- **THEN** Agent 调用 `json.to_query` 工具，dispatch 复用 `json_to_query_params` 的 `encoded` 字段返回 `aa=bb`，对话展示该结果

#### Scenario: Nested JSON flatten
- **WHEN** 用户请求将嵌套 JSON `{"a":{"b":1},"c":[10,20]}` 平铺
- **THEN** `json.flatten` 返回 `{"a[b]":"1","c[0]":"10","c[1]":"20"}`

#### Scenario: URL parsed into components
- **WHEN** 用户请求拆解 `https://api.x.com:8080/v1/x?k=v&k=w#frag`
- **THEN** `url.parse` 返回含 scheme=https / host=api.x.com / port=8080 / path=["v1","x"] / query={"k":["v","w"]} / fragment="frag" 的 JSON

#### Scenario: Form string parsed to object
- **WHEN** 用户请求把表单字符串 `aa=bb&x=hi%20world` 解析为对象
- **THEN** `form.parse` 返回 `{"aa":"bb","x":"hi world"}`（值已 URL 解码）

#### Scenario: New tools do not extend side effects
- **WHEN** 任一应用层工具被 Agent 调用
- **THEN** 调度链路不引入任何文件系统、网络或数据库副作用（与既有 12 工具同一 `SideEffect::None` 约束）

### Requirement: App-Layer Tool Dispatch Reuses Existing Commands
`json.to_query` 的 Agent dispatch MUST 复用既有 `commands::json::json_to_query_params` 函数（MUST NOT 重复实现 URL 编码逻辑）；前端 `JsonToQuery.vue` 继续调用原 command，原函数与契约 MUST 保持不变。

#### Scenario: Single source of truth
- **WHEN** 任一调用方（Agent dispatch 或前端 `JsonToQuery.vue`）请求 JSON→query
- **THEN** 两者走同一 Rust 函数，仅返回字段不同：前端取 `JsonToQueryResult` 双字段，Agent dispatch 取 `encoded` 单字符串

### Requirement: App-Layer Tool Schema Validity
4 个新增工具的 JSON Schema MUST 与既有 12 工具同一风格：`additionalProperties: false`、必填字段在 `required` 数组、类型用 `"string"`/`"integer"` 等基础类型。参数 schema 由 Agent 注册表托管，harness 校验（`validate_call`）自动覆盖；非法参数 MUST 被拒绝而不调用底层函数。

#### Scenario: Missing input rejected
- **WHEN** Agent 调用 `json.to_query` 但 `arguments` 缺少 `input` 字段
- **THEN** 注册表 schema 校验返回错误文本回填给模型，不调用 dispatch

#### Scenario: Type-mismatched input rejected
- **WHEN** Agent 调用 `url.parse` 但 `input` 不是字符串
- **THEN** 校验失败回填，工具不被调用