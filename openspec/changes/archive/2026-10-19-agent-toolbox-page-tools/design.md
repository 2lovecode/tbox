# Design

## Context

见 proposal.md。当前 `TOOLS` 约 18 个。多个 Skill 把页面能力标成「无 Agent tool」或「子集」。小模型提示把整份工具目录常驻写入，预算 6000 / 4500 字符。

## Goals / Non-Goals

**Goals:**

- 下表中的每个操作都能被 `<tool_call>` 调用，结果与页面对同一输入一致。
- 网络、数据库、密码库写入/读取必须先经现有审批闸门。

**Non-Goals:**

- 见 proposal 的 Non-goals。不在本设计里补 PDF 转码或真实文件恢复。

## Decisions

1. **一操作一工具 id，参数名与 command 对齐。** 不使用一个大工具加 `op` 枚举。小模型容易漏枚举；目录里一行「id：用途｜参数」更稳。已有 id 不改名：`encoding.convert` 仍是 URL 编码，另加 `encoding.url_decode`。`hash.digest` 的 `algorithm` 扩展为 `md5|sha1|sha256|sha384|sha512`，不新造 `hash.sha512`。`json.format` 描述改为只表示美化。
2. **复用 command。** dispatch 只取页面展示的那个字段。command 尚不存在、逻辑在 Vue 里的（坐标换算、编码页多数 tab、JSON diff、颜色的 HSL/HSV），把函数挪到 `commands`，页面改为 invoke。
3. **副作用。** 纯计算保持 `SideEffect::None`。`http.request`、`db.test`、密码库读写新增 `SideEffect::Confirm`，走与 `os.shell` 相同的审批后再执行。密码生成是纯计算，不审批。
4. **提示预算。** 目录保持一行一个工具。实现时按实际目录长度上调两个预算常量，测试改为「目录 + 少样本仍在预算内」，而不是把工具删出目录来迁就旧数字。
5. **分组落地。** 按领域分批注册，每批带 registry 测试与 Skill 样本。不在一批里改完全部页面。

### 要注册的操作

已有且保留：`json.format`、`json.to_query`、`json.flatten`、`base64.encode`、`base64.decode`、`hash.digest`（扩算法）、`jwt.parse`、`timestamp.convert`、`encoding.convert`、`xml.format`、`yaml.format`、`uuid.generate`（v4）、`cron.explain`、`number.convert`、`charset.convert`、`url.parse`、`form.parse`、`os.shell`。

新增（id → 页面操作）：

- JSON：`json.compress` `json.escape` `json.unescape` `json.validate` `json.info` `json.diff` `json.to_java` `json.to_csharp` `json.to_go` `json.to_python` `json.to_typescript`
- 编码：`encoding.url_decode` `encoding.unicode_encode` `encoding.unicode_decode` `encoding.base58_encode` `encoding.base58_decode` `encoding.hex_encode` `encoding.hex_decode` `encoding.html_encode` `encoding.html_decode` `encoding.punycode_encode` `encoding.punycode_decode` `encoding.binary_encode` `encoding.binary_decode` `encoding.morse_encode` `encoding.morse_decode` `encoding.rot13` `encoding.caesar` `encoding.case` `encoding.detect` `charset.detect` `charset.convert_text`（UTF-8↔GBK）
- 摘要与令牌：`jwt.generate` `gm.sm3` `gm.sm4_encrypt` `gm.sm4_decrypt` `gm.sm4_key` `gm.sm2_keypair` `gm.sm2_sign` `gm.sm2_verify` `gm.hmac_sm3`
- 时间与数值：`uuid.v7` `uuid.v5` `uuid.batch` `uuid.validate` `uuid.to_base64` `uuid.from_base64` `uuid.version` `uuid.nil` `cron.generate` `cron.validate` `cron.to_natural` `number.scientific` `number.roman` `number.fraction`
- 标记：`xml.minify` `xml.to_json` `json.to_xml` `xml.to_yaml` `yaml.to_xml` `yaml.validate` `yaml.to_json` `json.to_yaml` `yaml.merge`
- 开发：`code.format` `regex.test` `text.compare` `text.dedupe` `text.sort` `sql.format` `sql.minify` `sql.escape` `sql.unescape` `db.test` `csv.to_json` `json.to_csv` `csv.format` `csv.stats` `log.analyze` `log.count_errors` `log.filter` `log.levels`
- 媒体：`image.info` `image.to_base64` `qrcode.generate` `qrcode.parse` `barcode.generate`
- 设计：`color.rgb_hex` `color.to_hsl` `color.to_hsv` `color.random` `color.brightness` `geo.distance` `geo.convert`
- 网络：`http.request`
- 密码：`password.generate` `password.list` `password.get` `password.save` `password.delete`

## Risks / Trade-offs

- [Risk] 工具变多后小模型选错 id → Mitigation：Skill 样本与目录一行描述使用页面上的动词；评测保留「转义下 → json.escape」这类路由用例。
- [Risk] 常驻目录挤占上下文 → Mitigation：提高预算；Skill 正文仍检索装载。
- [Risk] 密码库与 HTTP 被模型误调用 → Mitigation：`SideEffect::Confirm`，未批准不执行。
- [Risk] 前端算法搬到 Rust 时行为漂移 → Mitigation：用页面当前样例做对照测试，再改 Vue 为 invoke。

## Migration Plan

分领域合并进注册表即可，无数据迁移。回滚时从 `TOOLS` 删除新增 id，Skill 改回引导页面。密码库若从 localStorage 迁到 Rust 可读存储，页面读写同一处，避免两份数据。
