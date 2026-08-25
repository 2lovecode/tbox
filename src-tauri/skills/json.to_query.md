---
tool_id: json.to_query
keywords: json, query, querystring, URL 参数, 转换, 转成 query, 转成 URL, form, 序列化
---

# JSON 转 URL Query

把 JSON 对象转为 URL-encoded query string（key/value 都做 percent-encoding）。
嵌套对象走 `a[b]`、数组走 `a[0]`/`a[1]`，与后端 PHP / qs 库风格一致。

参数：`input` — 待转换的 JSON 字符串（通常为对象）。

## 典型应用场景

- 把表单 JSON 序列化为 URL query string（GET 请求参数）
- 接口调试时把 JSON 体拼到 URL 上
- 与 `url.parse` 配合：先 `json.to_query` 再 `encoding.convert`（一般不需要，后者会重复编码）

## 用户问法 → 工具调用样本

**用户**：把 `{"aa":"bb"}` 转成 URL query string
**工具调用**：`<tool_call>{"name": "json.to_query", "arguments": {"input": "{\"aa\":\"bb\"}"}}</tool_call>`
**预期输出**：`aa=bb`

**用户**：把这个嵌套 JSON 转成 query：`{"a":{"b":"c"},"x":["1","2"]}`
**工具调用**：`<tool_call>{"name": "json.to_query", "arguments": {"input": "{\"a\":{\"b\":\"c\"},\"x\":[\"1\",\"2\"]}"}}</tool_call>`
**预期输出**：`a%5Bb%5D=c&x%5B0%5D=1&x%5B1%5D=2`（方括号被编码）

**用户**：convert JSON to query string: `{"name":"alice","age":30}`
**工具调用**：`<tool_call>{"name": "json.to_query", "arguments": {"input": "{\"name\":\"alice\",\"age\":30}"}}</tool_call>`
**预期输出**：`name=alice&age=30`

边界：非法 JSON 会报错；空对象返回空字符串。