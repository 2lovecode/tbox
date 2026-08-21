# BMAD → OpenSpec 迁移说明

**日期:** 2026-08-21

## 已删除

- `_bmad/`、`_bmad-output/`
- `.agents/`（BMAD/GDS/WDS skills）
- `.cursor/skills` 下原 BMAD skills（已替换为 Superpowers skills）

## 已迁入 OpenSpec living specs

| 原文档 | 新位置 |
|--------|--------|
| ROADMAP 产品定位 | `openspec/specs/product/spec.md` |
| 工具注册/路由惯例 | `openspec/specs/tool-registry/spec.md` |
| Epic 1 角色 | `openspec/specs/roles/spec.md` |
| Epic 2/3 Spotlight / 键盘流 | `openspec/specs/spotlight/spec.md` |
| `story-hardware-info.md` | `openspec/specs/hardware-info/spec.md` |
| Epic 5 本地智能搜索 | `openspec/specs/local-ai-search/spec.md` |
| IMPLEMENTATION_SUMMARY 编码加密 | `openspec/specs/encoding-crypto/spec.md` |

## 有意未整库搬迁

- `ROADMAP.md` 中未做完的 P0/P1/P2 清单：保留为 backlog，待真正开工时用 `/opsx-propose` 写成 delta，避免一次性堆出易过期的巨型 spec。
- `IMPROVEMENTS.md` / `IMPLEMENTATION_SUMMARY.md`：历史实现笔记；行为已收敛进上述 specs，原文可留档或继续 gitignore。

## 协作栈

OpenSpec + Superpowers（含 `openspec/schemas/superpowers-bridge`）。详见根目录 `AGENTS.md`。
