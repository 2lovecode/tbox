---
tool_id: uuid.generate
keywords: uuid, UUID, v4, 标识, ID, 生成, generate, guid
---

# 生成 UUID v4

## 何时使用 / 何时不用

- **用**：用户请求与本工具能力描述一致时
- **不用**：请求属于其它近邻工具或纯闲聊时，勿强行调用

`uuid.generate` 生成一个新的 UUID v4 字符串。**无参数**。

## 典型应用场景

- 测试用例 / 临时数据主键
- 文件上传时生成唯一文件名
- 分布式追踪 trace_id 临时生成（生产环境通常由服务端分配）

## 用户问法 → 工具调用样本

**用户**：生成一个 UUID
**工具调用**：`<tool_call>{"name": "uuid.generate", "arguments": {}}</tool_call>`
**预期输出**：形如 `f47ac10b-58cc-4372-a567-0e02b2c3d479` 的 36 字符字符串

**用户**：give me a uuid v4 please
**工具调用**：`<tool_call>{"name": "uuid.generate", "arguments": {}}</tool_call>`
**预期输出**：同上

边界：参数校验会拒绝任何额外字段（`additionalProperties: false`）；无需传参。