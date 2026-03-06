# TVP

本仓库包含：

- `assets/`：数据库与静态资源（substances/routes/dosages/DDI 等）
- `rust/`：Rust workspace（包含 `tvp_cli` / `tvp_engine` / `tvp_hrt_cli` / `tvp_hrt_engine`）

> Rust 项目的构建与运行请在 `rust/` 目录下执行。

## 两个命令行工具

- `tvp`（crate：`tvp_cli`）：用于 **PsychonautWiki 数据库**的查询/DDI 检查，以及 **pk/effect** 曲线与基于 events 的模拟。
- `tvp-hrt`（crate：`tvp_hrt_cli`）：用于 **HRT（Oyama-compatible）** 的 curve/simulate。

完整说明见：`rust/README.md`。
