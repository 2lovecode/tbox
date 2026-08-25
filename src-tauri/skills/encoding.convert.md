---
tool_id: encoding.convert
keywords: url, URL, encode, decode, percent, 百分号, 编码, 解码, urlencoder
---

# URL 编解码

`encoding.convert` 对字符串做 URL percent-encode（非字母数字字符全部转 `%XX`）。
参数：`input` — 待编码的字符串。

## 典型应用场景

- 把空格、中文、`#` `?` `&` 等字符编码进 URL
- 与 `url.parse` 配合：先 `url.parse` 拿到 `query` 字段，再单独编码某字段值
- **不要**对已经是 query string 的字符串重复编码（应直接用 `json.to_query`）

## 用户问法 → 工具调用样本

**用户**：URL 编码 `a b&c`
**工具调用**：`<tool_call>{"name": "encoding.convert", "arguments": {"input": "a b&c"}}</tool_call>`
**预期输出**：`a%20b%26c`

**用户**：urlencode `你好`
**工具调用**：`<tool_call>{"name": "encoding.convert", "arguments": {"input": "你好"}}</tool_call>`
**预期输出**：`%E4%BD%A0%E5%A5%BD`

**用户**：encode `#fragment` for URL
**工具调用**：`<tool_call>{"name": "encoding.convert", "arguments": {"input": "#fragment"}}</tool_call>`
**预期输出**：`%23fragment`

边界：空字符串返回空；纯 ASCII 字母数字保留。