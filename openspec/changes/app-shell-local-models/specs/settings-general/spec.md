## Purpose

设置页「通用」分区承载与产品壳相关的本机偏好，包括嵌入式引擎日志策略（自 LLM 分区迁入）。

## ADDED Requirements

### Requirement: General Settings Hosts Engine Log
设置页「通用」分区 SHALL 提供嵌入式引擎日志配置（最大文件大小、保留天数、是否镜像到标准输出、日志路径只读展示）。LLM 配置分区 MUST NOT 再重复承载该表单。配置持久化语义与既有 `llama_engine_log` 设置一致。

#### Scenario: Open general for engine log
- **WHEN** 用户打开设置并进入「通用」
- **THEN** 可见本地引擎日志相关控件并可保存

#### Scenario: LLM panel without engine log form
- **WHEN** 用户打开设置「LLM 配置」
- **THEN** 主视图不包含引擎日志大小/保留/stdout 表单
