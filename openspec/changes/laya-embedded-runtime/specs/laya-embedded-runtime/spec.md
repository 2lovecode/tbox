# Spec Delta

## Purpose

内置 Laya ONNX 推理运行时：把 laya 决策从外部 Python 服务迁移为 TBox 进程内的 ONNX 推理，并提供模型文件的下载、校验、删除生命周期管理，使用户零依赖获得智能路由能力。

## ADDED Requirements

### Requirement: Model Download
系统 SHALL 支持从配置的模型源（默认 Hugging Face，可选 hf-mirror 镜像）下载 Laya 模型文件（onnx 模型、tokenizer、校准配置）到应用数据目录；下载过程 SHALL 发送进度事件供界面展示；下载完成后 SHALL 校验文件完整性，失败时 SHALL 清理并允许重试。

#### Scenario: First-time download
- **WHEN** 用户首次切换到 embedded 模式且本地无模型文件
- **THEN** 系统开始下载并持续发出进度事件，界面展示进度，完成后模型可用于推理

#### Scenario: Download failure and retry
- **WHEN** 下载中断或校验失败
- **THEN** 系统清理不完整文件并在界面提供重试入口

#### Scenario: Delete model
- **WHEN** 用户在设置中删除已下载的模型
- **THEN** 模型文件被删除，embedded 模式回到「需要下载」状态，决策返回未命中

### Requirement: Embedded Inference
embedded 模式下系统 SHALL 在进程内完成分词、单次前向推理与温度校准，输出与外部服务等价的 choice 结果与置信度；推理失败时 SHALL 静默降级为未命中，不阻塞调用方。

#### Scenario: In-process decision
- **WHEN** 模式为 embedded、模型已下载，且收到路由请求
- **THEN** 系统进程内完成推理并按阈值返回命中或未命中，全程无外部网络请求

#### Scenario: Inference failure
- **WHEN** 模型未加载或推理出错
- **THEN** 决策结果为未命中，调用方回退路径不受影响

### Requirement: Numerical Parity Acceptance
内置推理 SHALL 通过与 Python 参考实现的一致性验收：在既有 50 条中文评测集上，choice 结果一致率 SHALL ≥ 0.9，且命中样本的置信度平均绝对偏差 SHALL ≤ 0.05。

#### Scenario: Parity eval
- **WHEN** 运行一致性验收（对齐既有评测集与 Python Router 输出）
- **THEN** 报告一致率与置信度偏差，未达标则不放开 embedded 模式为默认可选推荐

### Requirement: Mode Selection
系统 SHALL 提供决策模式配置：`off`（关闭，默认）、`embedded`（进程内推理）、`external`（外部 laya-serve）；切换模式 SHALL 即时生效且原有阈值、超时、降级语义不变。

#### Scenario: Switch to embedded
- **WHEN** 用户将模式从 off 切换到 embedded 且模型已就绪
- **THEN** 后续路由请求走进程内推理，不再请求外部 endpoint

#### Scenario: External mode unchanged
- **WHEN** 用户选择 external 模式
- **THEN** 行为与既有外部服务集成完全一致
