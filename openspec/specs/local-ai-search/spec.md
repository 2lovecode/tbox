# Local AI Search

## Purpose

本地智能搜索（Phase 1.5）。v0 已交付轻量本地能力；完整对话 / 嵌入式 LLM 由 `agent-chat` 与 `local-llm-runtime` 能力交付。

## Requirements

### Requirement: Phase 1.5 Lightweight Local Intelligence
系统 SHALL 提供不依赖云端大模型的本地检索增强，至少包括分词（jieba）与拼音匹配，并在 Spotlight 中可开关。

#### Scenario: Pinyin tool match
- **WHEN** 用户在启用本地智能搜索时用拼音片段检索工具
- **THEN** 能匹配到对应中文名称工具

#### Scenario: AI assist toggle
- **WHEN** 用户关闭 Spotlight 中的 AI/本地智能开关
- **THEN** 搜索回退到基础匹配行为

### Requirement: Spotlight Independent of Chat LLM
Spotlight 的本地智能搜索 MUST 保持不依赖对话 Agent 所用的 LLM 运行时。完整对话 / 嵌入式 LLM 由 `agent-chat` 与 `local-llm-runtime` 能力交付；使用 Phase 1.5 搜索时 MUST NOT 强制下载 GGUF 或启动 sidecar。

#### Scenario: No silent LLM dependency
- **WHEN** 用户仅使用 Spotlight 的 Phase 1.5 拼音/分词搜索
- **THEN** 该路径不强制下载模型，也不强制启动本地推理 sidecar
