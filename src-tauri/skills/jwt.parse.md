---
tool_id: jwt.parse
keywords: jwt, JWT, token, 令牌, 解码, decode, header, payload, 解析
---

# JWT 解析（不验签）

## 何时使用 / 何时不用

- **用**：用户请求与本工具能力描述一致时
- **不用**：请求属于其它近邻工具或纯闲聊时，勿强行调用

`jwt.parse` 把 JWT 字符串的 header 和 payload 部分做 Base64URL 解码并以 JSON
形式输出。**不校验签名**——只解析。参数：`input` — JWT 字符串（`header.payload.signature`）。

## 典型应用场景

- 调试接口时查看 Token 实际字段（exp / uid / scope 等）
- 排查 401：确认 Token 是否过期、签发方是否对
- 安全审计：检查 JWT 中是否泄露了敏感字段

## 用户问法 → 工具调用样本

**用户**：帮我解析这段 JWT `eyJhbGciOiJIUzI1NiJ9.e30.abc`
**工具调用**：`<tool_call>{"name": "jwt.parse", "arguments": {"input": "eyJhbGciOiJIUzI1NiJ9.e30.abc"}}</tool_call>`
**预期输出**：`{"header":{"alg":"HS256"},"payload":{},"verified":false}`

**用户**：decode this jwt: `eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.sig`
**工具调用**：`<tool_call>{"name": "jwt.parse", "arguments": {"input": "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.sig"}}</tool_call>`
**预期输出**：`{"header":{"alg":"HS256"},"payload":{"sub":"1"},"verified":false}`

边界：缺 segment 报错；segment 非 Base64URL 或非 JSON 报错；签名段不参与校验。