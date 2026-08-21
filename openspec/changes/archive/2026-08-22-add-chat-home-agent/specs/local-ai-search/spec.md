## RENAMED Requirements

- FROM: `### Requirement: Full LLM Deferred`
- TO: `### Requirement: Spotlight Independent of Chat LLM`

## MODIFIED Requirements

### Requirement: Spotlight Independent of Chat LLM
Spotlight 的本地智能搜索 MUST 保持不依赖对话 Agent 所用的 LLM 运行时。完整对话 / 嵌入式 LLM 由 `agent-chat` 与 `local-llm-runtime` 能力交付；使用 Phase 1.5 搜索时 MUST NOT 强制下载 GGUF 或启动 sidecar。

#### Scenario: No silent LLM dependency
- **WHEN** 用户仅使用 Spotlight 的 Phase 1.5 拼音/分词搜索
- **THEN** 该路径不强制下载模型，也不强制启动本地推理 sidecar
