## 1. Profile 生成参数（数据层）

- [x] 1.1 在 `LlmProfile` / `LlmProfileInput` / `LlmConfig` 增加可选 `temperature`、`top_p`、`max_tokens`、`n_ctx`（local），含 serde 默认与保存校验；用单元测试覆盖合法保存与非法 temperature 拒绝（`cargo test llm`）
- [x] 1.2 同步 `src/types/llm.ts` 与 `src/stores/llm.ts` 类型与 save 载荷；`vue-tsc --noEmit` 或项目既有类型检查通过

## 2. 设置 UI 与目录档位

- [x] 2.1 设置页 profile 编辑增加生成参数表单（清空=回退默认；local 显示 n_ctx）；手动路径：打开设置 → 编辑 profile → 保存后 list 回显
- [x] 2.2 `model_catalog` 增加 Agent 档位字段/文案（推荐 vs 轻量）；设置页列表展示；`cargo test model_catalog` 或相关断言通过

## 3. 推理链路读取参数

- [x] 3.1 embedded：`build_sampler` / `n_ctx` 读取激活 profile 参数，缺省回退硬编码默认；单测或集成断言自定义 temperature 路径被调用（`cargo test` 相关）
- [x] 3.2 云端/Ollama（genai 或现有 client）：补全请求带上 profile 的 temperature/top_p/max_tokens（不支持则降级不失败）；`cargo check -p tbox`（或 crate 名）通过

## 4. Harness 策略与提示

- [x] 4.1 `strategy_for` 按 catalog 档位区分完整强化 vs lite 强化；单测：recommended / lite / cloud（`cargo test harness`）
- [x] 4.2 `build_small_prompt` 固定「目录常驻 + Skill 按需」顺序，lite 档缩短 few-shot；预算断言按档位（`cargo test prompt`）

## 5. Skill 路由

- [x] 5.1 更新 16 个 `skills/*.md`：何时用/不用与近邻反例；keywords 偏路由
- [x] 5.2 `retrieve_skills` 近邻降权 + 单测（如 flatten 问法首位为 `json.flatten`）（`cargo test skills`）

## 6. Eval 归因

- [x] 6.1 实现首错类别枚举与规则归因；`EvalReport` 打印类别计数；`cases.json` 可选 `expect_failure_class`（`cargo test eval`）
- [x] 6.2 每类 ≥2 条轨迹前缀回归 case；逻辑层覆盖；有权重时抽跑 e2e 确认报告含归因（`cargo test` / 可选 `--features agent-eval`）

## 7. 收尾验证

- [x] 7.1 跑相关 `cargo test`（含 llm / harness / skills / eval）与前端类型检查；对照 delta specs 场景做一次核对清单
