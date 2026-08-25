---
tool_id: json.format
keywords: json, JSON, 格式化, 美化, pretty, format, 缩进
---

# JSON 格式化

`json.format` 把 JSON 字符串格式化为带缩进的可读形式。参数：`input` — 待格式化的 JSON 字符串。

## 典型应用场景

- 接口响应 / 配置文件可视化
- 排查 JSON 错误前先对齐格式
- 与 `json.to_query` / `json.flatten` 串联：先格式化看清结构再转换

## 用户问法 → 工具调用样本

**用户**：格式化这段 JSON `{"a":1}`
**工具调用**：`<tool_call>{"name": "json.format", "arguments": {"input": "{\"a\":1}"}}</tool_call>`
**预期输出**：
```
{
  "a": 1
}
```

**用户**：pretty print this json: `[1,2,3]`
**工具调用**：`<tool_call>{"name": "json.format", "arguments": {"input": "[1,2,3]"}}</tool_call>`
**预期输出**：带缩进的数组形式

边界：非法 JSON 报错并给出原因。