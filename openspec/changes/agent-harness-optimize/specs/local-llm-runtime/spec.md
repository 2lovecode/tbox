## ADDED Requirements

### Requirement: Agent Capability Labels On Catalog
精选本地模型目录中的每一项 SHALL 标明面向 Agent 的能力档位：至少区分「Agent 推荐」与「轻量（弱工具/闲聊）」；推荐项 MUST 为更适合工具调用的 Instruct 模型（本期 ≥1.5B），轻量项（如 0.5B）MUST NOT 显示为 Agent 推荐。设置页列表 MUST 展示该档位文案。

#### Scenario: 1.5B shown as agent recommended
- **WHEN** 用户打开本地模型目录
- **THEN** Qwen2.5 1.5B Instruct 条目显示为 Agent 推荐档，0.5B 条目显示为轻量档而非推荐

### Requirement: Embedded Sampling Uses Profile Generation Params
当后端为 embedded 且激活 profile 为 `local` 时，推理采样与上下文长度 MUST 使用该 profile 的生成参数（temperature、top_p、max_tokens、n_ctx）；字段缺失时 MUST 回退到引擎内置默认（与当前硬编码行为一致的合理默认：如 temperature 0.7、top_p 0.9、n_ctx 4096）。变更 n_ctx 后下次加载模型 MUST 生效。

#### Scenario: Custom temperature applied on local chat
- **WHEN** 用户将 local profile 的 temperature 设为 0.2 并保存，随后用 embedded 完成一轮对话
- **THEN** 该轮采样使用 temperature 0.2，而非忽略配置仍用旧硬编码且无回退说明

#### Scenario: Missing params fall back to defaults
- **WHEN** 旧版 profile JSON 无生成参数字段
- **THEN** embedded 推理仍可运行并使用内置默认采样与 n_ctx，不报错

## MODIFIED Requirements

### Requirement: Curated Model Download
系统 SHALL 在设置页提供精选模型目录（第一期 1～2 个小 Instruct GGUF），并标明 Agent 能力档位与推荐项。用户 MUST 能下载、看到进度、取消下载；只有校验完整的文件才可被标记为已安装并启用。下载位置 MUST 在应用数据目录（与现有 `~/.toolbox` 一类路径）。

#### Scenario: Download then enable
- **WHEN** 用户从设置页下载推荐模型且下载成功
- **THEN** 该模型显示为已安装，可被选为本地启用模型

#### Scenario: Failed download not installed
- **WHEN** 下载失败或用户取消
- **THEN** 该模型不得显示为已安装；不得留下可被启用的半截文件状态

#### Scenario: Offline chat after download
- **WHEN** 推荐模型已启用且嵌入式引擎可运行，用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM）
