---
tool_id: form.parse
keywords: form, form-urlencoded, x-www-form-urlencoded, 表单, 解析, 编码串, 序列化, query 字符串
---

# 表单字符串解析

把 `application/x-www-form-urlencoded` 字符串解析为 JSON 对象。重复键合并为
JSON 数组；值经 percent-decode；空字符串返回 `{}`。

参数：`input` — 表单编码字符串。

## 典型应用场景

- 解析抓包 / 日志里的 query string 到可读对象
- 接口调试：把 cURL `--data` 参数可视化
- 与 `json.to_query` 互逆（前者 JSON→query；本工具 query→JSON）

## 用户问法 → 工具调用样本

**用户**：解析这个表单 `aa=bb&x=hi%20world`
**工具调用**：`<tool_call>{"name": "form.parse", "arguments": {"input": "aa=bb&x=hi%20world"}}</tool_call>`
**预期输出**：`{"aa":"bb","x":"hi world"}`

**用户**：把 query string 转成对象 `a=1&a=2&b=3`
**工具调用**：`<tool_call>{"name": "form.parse", "arguments": {"input": "a=1&a=2&b=3"}}</tool_call>`
**预期输出**：`{"a":["1","2"],"b":"3"}`

**用户**：parse this form string: `flag`
**工具调用**：`<tool_call>{"name": "form.parse", "arguments": {"input": "flag"}}</tool_call>`
**预期输出**：`{"flag":""}`（无 `=` 视为 key，value 为空）

边界：空输入返回 `{}`；键或值 percent-decode 失败时返回错误。