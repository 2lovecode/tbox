---
name: JSON 工具集
description: JSON 美化/压缩/转义/验证/信息、转 query、平铺、转实体类、对比。XML/YAML 用 markup。
tool_id: json.format, json.to_query, json.flatten
toolbox_id: 9, 12, 13, 34
keywords: JSON, json, 格式化, 美化, 压缩, 转义, 反转义, 验证, query, querystring, 平铺, flatten, 实体类, Java, C#, Go, TypeScript, JSON对比, diff
avoid_keywords: XML, YAML, Base64, 哈希, JWT
---

# JSON 工具集

## 何时使用 / 何时不用

- **用**：处理 JSON（格式化/压缩/转义/验证、转 query、平铺、实体类、对比）
- **不用**：XML/YAML → `markup`；表单/URL 拆解 → `encoding`；通用代码美化 → `toolbox.dev`

## Agent 可调用

- `json.format` — 美化/规范化（也承接口语「转义/反转义」）；`input` = **用户 JSON 原文**
  - tool_call 信封只允许 **一层** 字符串转义；禁止先改成 `{\"a\":1}` 再塞进 input
  - 若工具提示「不要再多写反斜杠」→ 下一轮用原文重试，勿指责用户结构错误
- `json.to_query` — 对象 JSON → URL query；`input`
- `json.flatten` — 嵌套平铺为 `a[b]`/`c[i]`；`input`

Agent **不能**产出「带反斜杠的转义字符串」/压缩/验证/信息/实体类/diff → 开 `/json-tool` 等页面。

## 工具箱页面完整能力

- `/json-tool`(9)：美化（可设缩进）/ 压缩 / 转义 / 去转义 / 验证 / 信息
- `/json-to-entity`(12)：JSON → Java / C# / Go / Python / TypeScript
- `/json-diff`(13)：左右对比；过滤 全部/新增/删除/修改
- `/json-to-query`(34)：JSON → 未编码 query + URL 编码版

## 样本

**用户**：大段 JSON +「转义下」→ `json.format`，input 用原文（勿二次转义）  
**用户**：转 query / 平铺 → `json.to_query` / `json.flatten`  
**用户**：要 `{\"a\":1}` 这种转义串 / 转 Java / 对比 → 引导 `/json-tool`、`/json-to-entity`、`/json-diff`
