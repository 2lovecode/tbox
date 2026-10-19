# Tasks

## 1. 注册表与预算

- [x] 1.1 按 design.md 的 id 清单分批加入 `TOOLS` 与 dispatch；纯计算为 `SideEffect::None`，HTTP、数据库测试、密码库读写为需审批的副作用。`json.format` 只描述美化，`hash.digest` 扩展算法枚举。验证：每个新 id 能被 lookup，缺必填参数时不调用底层函数。
- [x] 1.2 上调小模型与轻量提示的字符预算，使常驻工具目录加少样本仍通过预算测试。验证：`prompt_within_budget` 与 lite 对应测试通过。

## 2. 与页面对齐

- [x] 2.1 已有 command 的操作只做 dispatch 包装。验证：registry 测试对 JSON 转义/去转义、XML 压缩、SQL 转义、UUID v7、Cron 校验至少各有一条，结果与直接调用 command 相同。
- [x] 2.2 目前只在 Vue 里的算法（编码页各 tab、坐标换算、颜色 HSL/HSV、JSON diff）抽到 Rust command，页面改为 invoke。验证：搬迁前后各用一组页面样例，输出一致。
- [x] 2.3 密码库读写与页面使用同一存储，并接入审批闸门。验证：未批准时不返回、不改动条目；批准后与页面看到的是同一条记录。

## 3. Skill 与回归

- [x] 3.1 更新 `src-tauri/skills/*.md` 与 `prompt.rs` 中把已注册操作引导去页面的句子和示例。验证：检索「转义 JSON」「SHA-512」「生成二维码」时，命中的 Skill 含对应工具 id，且不再写「请打开页面」。
- [x] 3.2 跑 registry、skills、prompt 相关 `cargo test`。验证：新增用例通过，既有 `json.to_query` 与 `uuid.generate` 用例仍通过。
