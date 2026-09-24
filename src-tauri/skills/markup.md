---
name: 标记语言格式化
description: XML 格式化/压缩与 XML↔JSON/YAML；YAML 格式化/验证/合并与 YAML↔JSON。JSON 用 json；通用代码美化用 toolbox.dev。
tool_id: xml.format, yaml.format
toolbox_id: 20, 21
keywords: XML, xml, YAML, yaml, 格式化XML, 格式化YAML, 压缩XML, XML转JSON, YAML转JSON, 合并YAML, 验证YAML
avoid_keywords: JSON格式化, JSON转义, 代码格式化通用
---

# 标记语言格式化

## 何时使用 / 何时不用

- **用**：XML/YAML 美化、压缩、互转、验证、合并
- **不用**：JSON → `json`；任意源码美化 → `toolbox.dev`

## Agent 可调用（子集）

- `xml.format` — **仅**弱格式化；`input`
- `yaml.format` — **仅**解析再序列化；`input`

Agent **不能**：XML 压缩、XML↔JSON/YAML、YAML 验证/合并/↔JSON（请开页面）。

## 工具箱页面完整能力

- `/xml-tools`(20)：格式化 / 压缩（`format_xml`/`minify_xml`）· XML↔JSON · XML↔YAML
- `/yaml-tools`(21)：格式化 + 验证（`format_yaml`/`validate_yaml`）· YAML↔JSON · 合并（`merge_yaml`）

## 样本

**用户**：格式化 XML `<a><b/></a>` → `xml.format`  
**用户**：XML 转 JSON / YAML 合并 → 引导 `/xml-tools` 或 `/yaml-tools`
