## 1. 日志后端

- [x] 1.1 新增 `llama_log` 模块：读/写 `llama_engine_log.json`、默认值、日志路径、轮转与按天清理；用单元测试覆盖默认反序列化与「超限应轮转」判定
- [x] 1.2 实现 `llama`/`ggml` 日志回调并在 `LlamaBackend::init` 后安装；默认只写文件；`mirrorStdout` 时镜像到 stderr；手动或集成路径确认控制台不再刷屏

## 2. 设置 API 与 UI

- [x] 2.1 注册 `get_llama_engine_log_settings` / `save_llama_engine_log_settings`（含 `logPath`）；`cargo test` 覆盖非法值钳制或报错
- [x] 2.2 `LlmSettingsPanel` 增加「本地引擎日志」表单（大小 MB、保留天、stdout 开关、路径只读）；保存后 `get` 回读一致

## 3. 验收

- [x] 3.1 `cargo test` 相关模块通过；对照 delta：默认无控制台噪声、开开关后有镜像、改大小/天数持久化
