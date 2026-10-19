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

### Requirement: Toolbox Page Operations Are Agent Tools
Agent 注册表 MUST 为每个工具页上有确定输入输出的操作提供工具 id。dispatch MUST 复用该操作现有的计算实现，使同一输入在页面与 Agent 得到相同结果。`side_effect` 为纯计算的操作 MUST 为 `None`。

至少覆盖这些原先不能调用的操作：

- JSON 页：压缩、转义、去转义、验证、信息；实体类（Java / C# / Go / Python / TypeScript）；JSON diff。`json.format` 只表示美化。
- 编码页：URL 解码、Unicode、Base58、十六进制、HTML 实体、Punycode、二进制、摩尔斯、ROT13、凯撒、大小写/反转、编码检测；字符集检测与 UTF-8↔GBK。`encoding.convert` 保持为 URL 编码。
- 摘要与令牌：`hash.digest` 支持 md5、sha1、sha256、sha384、sha512；JWT 生成；SM3、SM4 加解密与密钥、SM2 密钥对与签验、HMAC-SM3。
- 时间与数值：UUID v7、v5、批量、验证、与 Base64 互转、版本、NIL；Cron 生成、校验、自然语言；科学计数法、罗马数字、分数。`uuid.generate` 保持为单个 v4。`number.convert` 保持为进制转换。
- 标记语言：XML 压缩与 XML↔JSON/YAML；YAML 验证、合并与 YAML↔JSON。
- 开发页：代码格式化、正则测试、文本对比/去重/排序、SQL 格式化/压缩/转义/反转义、CSV↔JSON/格式化/统计、日志分析/错误统计/过滤/级别提取、数据库连接测试。
- 媒体：图片信息、图片转 Base64、二维码生成与解析、条码生成。
- 设计：RGB↔HEX、RGB→HSL、RGB→HSV、随机色、亮度、坐标距离、坐标系互转。
- 网络：HTTP 请求。
- 密码：随机密码生成，以及本地密码库的列出、读取、保存、删除。

屏幕标尺拖拽、坐标画布、图片压缩画布、网速模拟、视频模拟转换、未接通的 PDF 处理、演示用文件恢复 MUST NOT 注册为会假装完成这些事的工具。

#### Scenario: Escape uses the escape tool
- **WHEN** 用户要求把合法 JSON「转义」
- **THEN** Agent 可调用的工具是 `json.escape`，返回与页面「转义」相同的文本，而不是 `json.format` 的美化结果

#### Scenario: SHA-512 is selectable
- **WHEN** 用户要求计算文本的 SHA-512
- **THEN** `hash.digest` 接受 `algorithm=sha512` 并返回与哈希页相同的摘要

#### Scenario: Page and agent share one result
- **WHEN** 对任一新注册操作，页面与 Agent 使用同一输入
- **THEN** 两者输出一致

### Requirement: Sensitive Page Tools Require Approval
访问网络、测试数据库连接，或读取、写入、删除本地密码库的 Agent 工具 MUST 在用户批准后才执行。未批准时 MUST NOT 发出请求或改动密码库。随机密码生成不在此列。

#### Scenario: HTTP waits for approval
- **WHEN** Agent 调用 HTTP 请求工具且用户尚未批准
- **THEN** 请求未发出，对话停留在待批准状态

#### Scenario: Password vault waits for approval
- **WHEN** Agent 调用列出或读取本地密码且用户尚未批准
- **THEN** 不返回已保存的密码