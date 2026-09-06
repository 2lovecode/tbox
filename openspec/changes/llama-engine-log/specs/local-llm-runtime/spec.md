## ADDED Requirements

### Requirement: Embedded Engine Log Isolation
嵌入式 llama.cpp / ggml 运行日志 SHALL 默认写入应用数据目录下的独立日志文件（`~/.toolbox/logs/llama-engine.log` 或等价路径），MUST NOT 默认打印到进程标准输出或标准错误。仅当用户在设置中开启「同时输出到标准输出」时，MUST 将同一日志镜像到标准输出或标准错误。应用自身业务日志（非 llama/ggml）不受本要求约束。

#### Scenario: Default no console spam
- **WHEN** 用户加载本地嵌入式模型且未开启「输出到标准输出」
- **THEN** 控制台不出现 `load_tensors` / CUDA Graph 一类 llama.cpp INFO 行，且日志文件中可找到对应内容

#### Scenario: Opt-in console mirror
- **WHEN** 用户开启「同时输出到标准输出」后再次触发引擎日志
- **THEN** 日志仍写入文件，并同时出现在标准输出或标准错误

### Requirement: Engine Log Retention Policy
系统 SHALL 对嵌入式引擎日志文件实施可配置的大小上限与保留天数：超过大小上限 MUST 轮转（保留有限历史文件）；超过保留天数的轮转文件 MUST 被清理。未配置时 MUST 使用默认策略（大小上限 5MB、保留 7 天）。设置页 LLM 分区 SHALL 提供上述两项与「输出到标准输出」开关的编辑，并展示当前日志文件路径（只读）。配置 MUST 持久化到本地磁盘，重启后仍生效。

#### Scenario: Configure limits in settings
- **WHEN** 用户在设置「本地引擎日志」中将最大大小改为 10MB、保留天数改为 3 并保存
- **THEN** 随后的轮转与清理按新值执行，重启应用后设置仍为 10MB / 3 天

#### Scenario: Rotate when over size
- **WHEN** 当前日志文件写入后超过配置的大小上限
- **THEN** 系统轮转该文件并继续写入新的当前日志文件

#### Scenario: Purge expired rotations
- **WHEN** 存在早于保留天数的轮转日志文件且发生写日志或引擎启动清理
- **THEN** 这些过期文件被删除
