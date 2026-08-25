---
tool_id: yaml.format
keywords: yaml, YAML, 格式化, 美化, pretty, format
---

# YAML 格式化

`yaml.format` 校验并美化 YAML 字符串（缩进 / 空行）。参数：`input` — 待格式化的 YAML。

## 典型应用场景

- Kubernetes manifest / docker-compose / GitHub Actions 工作流可视化
- 把压在一行的多文档 YAML 拆开
- 与 `json.format` 配合：JSON ↔ YAML 互转（先 JSON→结构化再 YAML 写出）

## 用户问法 → 工具调用样本

**用户**：格式化 YAML `a: 1`
**工具调用**：`<tool_call>{"name": "yaml.format", "arguments": {"input": "a: 1"}}</tool_call>`
**预期输出**：
```
a: 1
```

**用户**：format this yaml: `name: tbox`
**工具调用**：`<tool_call>{"name": "yaml.format", "arguments": {"input": "name: tbox"}}</tool_call>`
**预期输出**：
```
name: tbox
```

边界：非法 YAML 报错并给出原因。