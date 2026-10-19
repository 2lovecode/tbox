# Spec Delta

## ADDED Requirements

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
