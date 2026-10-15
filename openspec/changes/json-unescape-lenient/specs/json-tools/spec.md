# Spec Delta

## Purpose

JSON 处理工具（美化 / 压缩 / 转义 / 去转义 / 验证 / 信息）的本地行为契约；本 change 先落地去转义的宽松解层要求。

## ADDED Requirements

### Requirement: JSON Unescape Lenient
JSON 去转义 SHALL 将输入视为可能经过一层或多层 JSON 字符串转义的文本，按 JSON 字符串字面量规则解层（MUST 正确处理 `\"`、`\\`、`\n`/`\r`/`\t`、`\uXXXX` 等），输出还原后的字符串且 MUST NOT 自动美化。返回结果 MUST 包含是否可解析为合法 JSON 的有效性标志。

#### Scenario: Single-escaped object with unicode escapes
- **WHEN** 用户对形如 `{\"a\":\"\\u0026\"}` 的转义串执行去转义
- **THEN** 输出为可解析的 JSON 文本（例如含 `"a":"&"` 或等价 `\u0026` 已按 JSON 规则还原的内容），且有效性标志为真，输出未经 pretty 缩进

#### Scenario: ES query with double-escaped newlines in script source
- **WHEN** 用户粘贴含 `\\n` 与 `\\u0026\\u0026` 的整段转义 ES query JSON 并执行去转义
- **THEN** 输出为可解析的 JSON 对象文本，script `source` 内换行与 `&&` 语义正确，有效性标志为真

#### Scenario: Already-plain JSON
- **WHEN** 用户对未转义的合法 JSON 对象执行去转义
- **THEN** 输出与输入语义等价（允许仅空白差异），有效性标志为真
