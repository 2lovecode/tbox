---
tool_id: json.flatten
keywords: json, flatten, 平铺, 嵌套, nested, 路径, path, dot, 对象转路径
---

# 嵌套 JSON 平铺

把任意嵌套 JSON 展平为 `{path: value}` 单层对象。嵌套对象路径用 `[key]`，
数组下标用 `[i]`。返回 JSON 对象，值为字符串（标量转字符串保留）。

参数：`input` — 待平铺的 JSON 字符串。

## 典型应用场景

- 把配置 JSON 转为「点路径键」形式用于环境变量或 Redis 存储
- 接口字段扁平化后比对
- 与 `json.to_query` 路径风格一致，但保留为 JSON 而非 URL 编码

## 用户问法 → 工具调用样本

**用户**：把嵌套 JSON `{"a":{"b":1}}` 平铺
**工具调用**：`<tool_call>{"name": "json.flatten", "arguments": {"input": "{\"a\":{\"b\":1}}"}}</tool_call>`
**预期输出**：`{"a[b]":"1"}`

**用户**：flatten this JSON: `{"a":{"b":1},"c":[10,20]}`
**工具调用**：`<tool_call>{"name": "json.flatten", "arguments": {"input": "{\"a\":{\"b\":1},\"c\":[10,20]}"}}</tool_call>`
**预期输出**：`{"a[b]":"1","c[0]":"10","c[1]":"20"}`

**用户**：把 API 返回的嵌套 JSON 转成点路径键值对
**工具调用**：见样本（同一 `json.flatten`）

边界：空对象返回 `{}`；标量值始终序列化为字符串。