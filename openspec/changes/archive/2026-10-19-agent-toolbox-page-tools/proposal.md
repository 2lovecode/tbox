# Proposal

## Why

工具箱页面上的计算能力大多已经实现，但 Agent 白名单只注册了其中一小部分。Skill 还明确写着「其余请打开页面」。用户在对话里说出页面上已有的操作时，模型无法调用，只能被引导去点界面。

## What Changes

- 每个工具页上**有确定输入输出的操作**都注册为 Agent 工具，dispatch 复用现有 command 或与页面相同的实现。已有工具（如 `json.format`、`base64.encode`）保留，缺的操作补 id。
- 口语与页面标签对齐：例如「转义」不再走美化，哈希不再只有 md5/sha256，UUID 不再只有单个 v4。
- 更新全部相关 Skill：删掉「Agent 不能 / 请开页面」里那些已经能调用的操作。
- 小模型常驻工具目录变长。提高 `SMALL_PROMPT_CHAR_BUDGET` 与 `LITE_PROMPT_CHAR_BUDGET`，使一行一个工具仍能放下。Skill 正文仍按检索取前几条，不整本塞进提示。
- 会访问网络、数据库或本地密码库的操作走审批，不静默执行。

## Non-goals

下列没有可复用的计算结果，不注册成工具，Skill 继续说明边界：

- 纯界面：屏幕标尺拖拽、坐标可视化画布、图片压缩画布、复制/示例/导入导出对话框、ASCII 码表展示。
- 演示或未接通：网速测试（随机数）、视频转换（模拟进度）、PDF 合并分割（后端未接通）、文件恢复（假数据）。

不改各页面的交互布局。

## Capabilities

### New Capabilities

- 无

### Modified Capabilities

- `app-layer-tools-registered`: 注册表覆盖各工具页可计算操作，而不只是原先 4 个应用层工具。
- `skill-content-enrichment`: Skill 把页面操作映射到对应工具 id，不再把已注册操作写成「请打开页面」。

## Impact

- Rust：`src-tauri/src/agent/registry.rs` 注册与 dispatch；`src-tauri/src/agent/harness/prompt.rs` 预算与示例；各 `commands` 只被调用，页面契约不变。页面逻辑若只在前端（坐标换算、部分编码），抽到 Rust 后页面改为 invoke，避免两套算法。
- Skill：`src-tauri/skills/*.md`。
- 前端：计算仍走原 command；仅当算法目前只在 Vue 里时，页面改为调用新 command。
