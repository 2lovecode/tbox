---
tool_id: json.flatten
keywords: flatten, 平铺, 嵌套, nested, 展平, 路径, 对象转路径
avoid_keywords: query string, URL 参数, 转成 query, form-urlencoded
---

# 嵌套 JSON 平铺

把任意嵌套 JSON 展平为 `{path: value}` 单层对象。嵌套对象路径用 `[key]`，
数组下标用 `[i]`。返回 JSON 对象，值为字符串（标量转字符串保留）。

参数：`input` — 待平铺的 JSON 字符串。

## 何时使用 / 何时不用

- **用**：用户要「平铺 / flatten / 展平嵌套 JSON」得到 JSON 对象
- **不用**：要 URL query string / 表单编码 → 用 `json.to_query`

## 典型应用场景

- 把配置 JSON 转为路径键形式
- 接口字段扁平化后比对

## 用户问法 → 工具调用样本

**用户**：把嵌套 JSON `{"a":{"b":1}}` 平铺
**工具调用**：`<tool_call>{"name": "json.flatten", "arguments": {"input": "{\"a\":{\"b\":1}}"}}</tool_call>`
**预期输出**：`{"a[b]":"1"}`

**用户**：flatten this JSON: `{"a":{"b":1},"c":[10,20]}`
**工具调用**：`<tool_call>{"name": "json.flatten", "arguments": {"input": "{\"a\":{\"b\":1},\"c\":[10,20]}"}}</tool_call>`
**预期输出**：`{"a[b]":"1","c[0]":"10","c[1]":"20"}`

边界：空对象返回 `{}`；标量值始终序列化为字符串。
