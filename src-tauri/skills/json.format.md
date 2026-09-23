---
tool_id: json.format
keywords: 格式化, 美化, pretty, format, 缩进, 排版 JSON, 转义, 反转义, 去转义, unescape, escape, 转义一下, 转义下
avoid_keywords: 字符集, charset, UTF-8, GBK, 乱码
---

# JSON 格式化

## 何时使用 / 何时不用

- **用**：「转义 / 反转义 / 去转义 / 格式化 / 美化」JSON（含 `{\"a\":1}`）
- **不用**：字符集/乱码 → `charset.convert`；转 query → `json.to_query`
- **禁止**：把「转义」当成 `charset.convert` 或 URL 编码

本工具 = 反转义 + 缩进美化（无单独 unescape）。回复直接展示工具返回的缩进 JSON。

**用户**：转义下 `{\"a\":1}`
**调用**：`<tool_call>{"name":"json.format","arguments":{"input":"{\"a\":1}"}}</tool_call>`
**禁止**：`{\\\"a\\\":1}`（多套反斜杠）；禁止改调 `charset.convert`

规则：用户原文原样进 input；报错仍用原文重试。
