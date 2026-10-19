# Zero-shot 评测结论（任务 1.3）

- 日期：2026-02（apply 阶段实测）
- checkpoint：`laya-multilingual`（mmBERT-base，max_len=8192），zero-shot 未微调
- 评测集：`queries.json`，50 条中文自然语言查询 → 36 个已注册工具
- 环境：macOS（Metal），仓库内 `.venv-laya`，延迟为单次 `Router.predict`（含 36 选项 choice）

## 结果

| 指标 | 数值 |
|------|------|
| choice 准确率（不看阈值） | **30/50 = 0.600** |
| 延迟 p50 / p95 | 32ms / 41ms |
| 置信度校准质量 | 偏保守：约 1/5 答对的样本 conf < 0.5 |

### 阈值扫描（精度 = 命中且正确 / 触发置顶）

| threshold | 覆盖 | 精度 |
|-----------|------|------|
| 0.2 | 46/50 | 0.652 |
| 0.3 | 38/50 | 0.737 |
| 0.4 | 34/50 | 0.794 |
| **0.5** | **31/50** | **0.839** |
| 0.6 | 25/50 | 0.880 |
| 0.7 | 24/50 | 0.875 |

## 决定

- choice 准确率 0.600 ≥ 门槛 0.6 → **按计划继续实现**（功能仍默认关闭）。
- 默认阈值从提案的 0.7 调整为 **0.5**：0.7 覆盖仅 24/50 且精度无提升（0.875 vs 0.839），
  0.5 在覆盖与精度之间最优。spec 增量已同步更新。
- 置顶推荐的期望精度约 0.84（约每 6 次置顶 1 次不准），对「置顶 + 关键词结果仍可见」的
  交互可接受；若需更高精度，下一 change 做 RLCD 微调（官方微调后 typed-decisions 0.766 vs 0.362）。

## 复现

```bash
HF_HOME=.venv-laya/hf HF_ENDPOINT=https://hf-mirror.com HF_HUB_DISABLE_XET=1 \
  .venv-laya/bin/python openspec/changes/laya-spotlight-tool-routing/eval/run_eval.py
```
