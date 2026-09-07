## Context

`agent::skills` 通过 `include_str!` 内置 16 个 Markdown Skill，按用户问题检索后注入系统提示。现有设置页已有分区壳，但没有 Skill 分区；Skill 数据也没有用户偏好。

## Goals / Non-Goals

**Goals:** 查看内置 Skill 目录；按项启停；检索过滤；本机持久化。

**Non-Goals:** 不做第三方 Skill 安装、正文编辑、导入导出或工具卸载。

## Decisions

### 1. 内置目录、用户目录与布尔偏好

- **选择**：Rust 继续持有内置 Markdown 作为内置目录；用户创建/修改的 Skill 与导入的外部 Markdown 写入 `~/.toolbox/skills/custom/`；偏好文件继续保存禁用集合。默认全部启用，缺失项视为启用。
- **理由**：内置文档随应用升级保持一致；用户数据与程序数据分离；Markdown 目录足够薄，适合渐进式披露。

### 2. 统一检索和安全边界

- **选择**：`retrieve_skills` 合并内置与用户 Skill，在评分前跳过禁用项；所有 Skill 只作为说明书注入，可关联注册表工具，但导入/创建 Skill 不新增 dispatch 入口。
- **理由**：所有 Agent 后端都经过统一 harness，过滤入口越靠前越能节省上下文预算；同时避免外部 Markdown 变成任意代码或工具注册通道。
- **理由**：所有 Agent 后端都经过统一 harness，过滤入口越靠前越能节省上下文预算。

### 3. 设置页独立分区

- **选择**：新增 `/settings/skills` 与 `SkillSettingsPanel`，而不是塞入通用设置。
- **理由**：Skill 是 Agent 能力目录，后续可能扩展依赖、版本和作用域，独立分区比通用偏好更清晰。
