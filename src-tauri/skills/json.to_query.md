---
tool_id: json.to_query
keywords: query, querystring, URL 参数, 转成 query, 转成 URL, form, 序列化 query
avoid_keywords: 平铺, flatten, 嵌套平铺, 展平
---

# JSON 转 URL Query

把 JSON 对象转为 URL-encoded query string（key/value 都做 percent-encoding）。
嵌套对象走 `a[b]`、数组走 `a[0]`/`a[1]`，与后端 PHP / qs 库风格一致。

参数：`input` — 待转换的 JSON 字符串（通常为对象）。

## 何时使用 / 何时不用

- **用**：用户要「转成 query string / URL 参数 / form 编码串」
- **不用**：只要把嵌套 JSON **平铺成 JSON 对象**（不编码 URL）→ 用 `json.flatten`；只要拆 URL → 用 `url.parse`

## 典型应用场景

- 把表单 JSON 序列化为 URL query string（GET 请求参数）
- 接口调试时把 JSON 体拼到 URL 上

## 用户问法 → 工具调用样本

**用户**：把 `{"aa":"bb"}` 转成 URL query string
**工具调用**：`<tool_call>{"name": "json.to_query", "arguments": {"input": "{\"aa\":\"bb\"}"}}</tool_call>`
**预期输出**：`aa=bb`

**用户**：convert JSON to query string: `{"name":"alice","age":30}`
**工具调用**：`<tool_call>{"name": "json.to_query", "arguments": {"input": "{\"name\":\"alice\",\"age\":30}"}}</tool_call>`
**预期输出**：`name=alice&age=30`

边界：非法 JSON 会报错；空对象返回空字符串。勿与「平铺嵌套 JSON」混淆。
