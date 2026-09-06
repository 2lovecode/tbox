---
tool_id: base64.encode, base64.decode
keywords: base64, Base64, 编码, 解码, encode, decode
---

# Base64 编解码

## 何时使用 / 何时不用

- **用**：用户请求与本工具能力描述一致时
- **不用**：请求属于其它近邻工具或纯闲聊时，勿强行调用

`base64.encode` 将字符串做标准 Base64 编码；`base64.decode` 反向解码（结果须为 UTF-8）。
参数：`input` — 待编码/解码的字符串。

## 典型应用场景

- 接口调试、Token / Cookie 字段可视化
- 二进制摘要的可读化（一般先 `hash.digest` 再 `base64.encode`）

## 用户问法 → 工具调用样本

**用户**：把 `hi` 编成 Base64
**工具调用**：`<tool_call>{"name": "base64.encode", "arguments": {"input": "hi"}}</tool_call>`
**预期输出**：`aGk=`

**用户**：解码 `aGk=`
**工具调用**：`<tool_call>{"name": "base64.decode", "arguments": {"input": "aGk="}}</tool_call>`
**预期输出**：`hi`

**用户**：base64 encode `hello world`
**工具调用**：`<tool_call>{"name": "base64.encode", "arguments": {"input": "hello world"}}</tool_call>`
**预期输出**：`aGVsbG8gd29ybGQ=`

边界：解码结果非 UTF-8 时报错。