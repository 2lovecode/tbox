## Context

Laya 的 `state` 在候选项之后截断，长 JSON 会挤掉「转义 / 去转义」。`RouteHit.tool_id` 是工具箱页面的数字 id，不是 `json.escape` 这类 agent 工具 id。Agent 循环跑在 `spawn_blocking` 里。

## Goals / Non-Goals

**Goals:**

- 规则压缩决策状态，再调用现有 `route_embedded` / external `/v1/systemone`。
- 仅当置信度达到设置里的阈值时，把页面名写入系统提示，并把该页技能登记的工具 id 排到目录前。
- 模型仍输出 `<tool_call>`，参数仍来自用户原文。

**Non-Goals:**

- 不用 qwen 写摘要。
- 不让 Laya 在全部 agent 工具 id 上做一次选择。
- 不改变 Spotlight 的置顶逻辑。
- 低置信度不注入反向提示。

## Decisions

1. 压缩规则：丢掉以 `{` / `[` 开头或明显是转义 JSON 的行，留下意图句；若因此删过载荷，句尾标「有一段JSON」。仅当用户原文本身不超过 40 字时，才附上历史里最近一条工具名和结果首行。总长截到 180 字。
2. 页面 id 经技能 frontmatter 的 `toolbox_id` 映射到 agent 工具 id。映射失败时仍写页面名参考，不改目录。
3. 嵌入式推理直接调用同步 `route_embedded`。外部模式用 `block_on`。任何失败视为未命中。
4. 单测构建里 Agent 循环不读本机 Laya 配置，避免用例去加载 ONNX。决策函数本身用传入的 settings 测试关闭分支。

## Risks / Trade-offs

- 一个技能覆盖多个页面（例如 JSON 的 9/12/13/34）时，排到前面的是该技能的全部工具，比单个页面宽。模型仍可选用目录里靠后的工具。
- 外部 laya-serve 在阻塞线程里 `block_on`，与现有 HTTP 工具相同。
