---
tool_id: url.parse
keywords: url, URL, 解析, parse, 拆解, scheme, host, port, path, query, fragment, 域名
---

# URL 拆解

## 何时使用 / 何时不用

- **用**：用户请求与本工具能力描述一致时
- **不用**：请求属于其它近邻工具或纯闲聊时，勿强行调用

把 URL 字符串解析为结构化 JSON（`scheme` / `host` / `port` 可选 / `path` 段数组 /
`query` 对象 / `fragment` 可选）。query 重复键合并为 JSON 数组；query 值经
percent-decode。

参数：`input` — URL 字符串。

## 典型应用场景

- 在不改请求库的情况下提取某 URL 的 host / path / 单个 query 参数
- 接口调试日志展示结构化 URL
- 与 `form.parse` 配合：`url.parse` 后用 `query` 字段继续处理

## 用户问法 → 工具调用样本

**用户**：拆解这个 URL：`https://api.x.com:8080/v1/x?k=v&k=w#frag`
**工具调用**：`<tool_call>{"name": "url.parse", "arguments": {"input": "https://api.x.com:8080/v1/x?k=v&k=w#frag"}}</tool_call>`
**预期输出**：`{"scheme":"https","host":"api.x.com","port":8080,"path":["v1","x"],"query":{"k":["v","w"]},"fragment":"frag"}`

**用户**：parse URL `https://x.com/path?token=hi%20world`
**工具调用**：`<tool_call>{"name": "url.parse", "arguments": {"input": "https://x.com/path?token=hi%20world"}}</tool_call>`
**预期输出**：`{"query":{"token":"hi world"}}`（query 值已解码）

**用户**：这个 URL 的 host 是什么 `http://example.com/foo`
**工具调用**：`<tool_call>{"name": "url.parse", "arguments": {"input": "http://example.com/foo"}}</tool_call>`
**预期输出**：`{"host":"example.com","path":["foo"]}`

边界：无 host 的非法 URL 报错；无 `port` / 无 `fragment` 时字段省略。