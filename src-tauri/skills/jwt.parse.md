---
tool_id: jwt.parse
keywords: jwt, JWT, token, 令牌, 解析, payload, header, 验签
---

# JWT 解析（不验签）

使用 `jwt.parse` 解析 JWT 的 header 与 payload JSON。

**重要：只解析结构，不验证签名。** 输出中 `verified` 恒为 `false`；不可用于鉴权决策。

参数：`input` — 完整 JWT 字符串（`header.payload[.signature]`）。

适用：调试 token 内容、查看 claims、检查过期字段等只读场景。
