# TVP (Rust workspace)

> 说明：本 workspace 的 Cargo 根目录在 `rust/`。运行命令时请在该目录下执行（例如 `cd rust` 后再跑 `cargo test`）。

## 1) 两个命令行工具

本 workspace 主要有两个可执行程序（两个“命令”）：

1. `tvp`（crate：`tvp_cli`）
	 - 面向 PsychonautWiki 风格数据库（`assets/database`）
	 - 功能：db 查询、DDI 检查、PK 曲线、Journal 风格 effect timeline、基于 events 的模拟

2. `tvp-hrt`（crate：`tvp_hrt_cli`）
	 - 面向 HRT 数据库（`assets/database/hrt`）
	 - 功能：Oyama-compatible 的 curve/simulate（事件驱动）

### 构建与运行

在 `rust/` 目录下：

- 查看帮助：
	- `cargo run -p tvp_cli -- --help`
	- `cargo run -p tvp_hrt_cli -- --help`

- 运行子命令：
	- `cargo run -p tvp_cli -- <subcommand> ...`
	- `cargo run -p tvp_hrt_cli -- <subcommand> ...`

> 注意：这里故意不写“固定二进制名称”的运行方式（例如直接执行 `tvp.exe`），因为你的构建产物路径会随 profile/target 变化；用 `cargo run -p ...` 最稳定。

## 2) CLI 输出格式：Envelope v1

本仓库的两个 CLI：
- `tvp_cli`（pk / effect / ddi / db）
- `tvp_hrt_cli`（HRT / Oyama-compatible）

统一输出顶层 envelope：

- `schema`: 固定为 `tvp.cli.envelope.v1`
- `engine`: 产生结果的后端（例如 `tvp_engine.pk` / `tvp_engine.effect` / `tvp_hrt_engine.sim`）
- `command`: CLI 子命令（例如 `curve` / `simulate`）
- `db_dir`: 实际使用的数据库目录
- `subject_kind`: 例如 `substance` / `compound`
- `subject` / `route` / `dosage`: 若能从 CLI 参数或 DB 解析则填充
- `events`: **对象**，形如 `{ "meta": {...}, "events": [...] }`
- `result`: 引擎输出
- `warnings` / `notes`: 可选

### events 约定

`events.meta` 用于携带本次模拟/曲线的配置（网格、时长、输入文件路径等）。
`events.events` 是事件数组（原始输入或规范化后的事件）。

## 3) tvp（tvp_cli）用法

### 3.1 数据库路径

多数命令支持 `--db-dir`，默认值为 `assets/database`。
程序会在相对路径无效时自动探测 `../assets/database` 等。

### 3.2 子命令速览

- `tvp db stats`：数据库统计
- `tvp db get <substance>`：按名称/id/commonName 查询物质
- `tvp ddi check --substances "A,B,C"`：DDI 检查
- `tvp pk curve ...`：Bateman PK 单次曲线
- `tvp effect curve ...`：Journal 风格 effect timeline 曲线（durations + strength + convolution）
- `tvp simulate --kind effect|pk --events <path> ...`：从 events JSON 驱动模拟

### 3.3 effect curve（Journal 风格时间线）

用途：复刻 psychonautwiki-journal-android 的“完整模拟”时间线语义（与浓度 PK 不同）。

常用参数：

- `--substance <name|id>`：物质
- `--compound <name|id>`：别名输入（等价于 substance；优先用 compound）
- `--route <name|id>`：途径
- `--dose <f64>`：剂量（单位取决于 L3.dose_units）
- `--t0-h <f64>`：开始时间（小时，默认 0）
- `--t1-h <f64>`：可选结束时间（给定则视为 time-range ingestion）
- `--total-h <f64>` / `--steps <usize>`：输出曲线网格

示例：pregabalin 口服 225mg，输出 24h：

- `cargo run -p tvp_cli -- effect curve --substance pregabalin --route oral --dose 225 --total-h 24 --steps 400`

### 3.4 simulate（events 驱动）

#### 3.4.1 --kind=effect 的 events 文件格式

events JSON 必须是数组，每项形如：

```json
{
	"t0_h": 0.0,
	"t1_h": null,
	"dose": 225.0,
	"horizontal_weight": 0.5,
	"common_dose": 225.0
}
```

- `t1_h` 为 null 表示点摄入；为数字表示 time-range ingestion。
- `common_dose` 省略时会回退为本次 `dose`（使单次摄入时 height=1）。

运行：

- `cargo run -p tvp_cli -- simulate --kind effect --events tmp_effect_events.json --substance pregabalin --route oral --total-h 24 --steps 400`

#### 3.4.2 --kind=pk 的 events 文件格式

events JSON 必须是数组，每项形如：

```json
{ "time_h": 0.0, "dose": 225.0 }
```

含义：在 `time_h` 时刻给 `dose`（与单次 Bateman 响应线性叠加）。

运行：

- `cargo run -p tvp_cli -- simulate --kind pk --events tmp_pk_events.json --substance pregabalin --route oral --dt-minutes 2 --total-h 24`

## 4) tvp-hrt（tvp_hrt_cli）用法

### 4.1 数据库路径

默认 `--db-dir assets/database/hrt`，并支持自动探测 `../assets/database/hrt`。

### 4.2 curve

用途：从 HRT L1/L2/L3 配置构造事件并输出固定网格曲线（统一 envelope 输出）。

常用参数：

- `--compound <name|id>`：必填
- `--substance <name|id>`：可选别名输入（优先使用 substance）
- `--route <name|id>`：必填
- `--dose <f64>`：剂量（单位取决于 HRT L3.dose_units）
- `--t0-h <f64>` / `--total-h <f64>` / `--steps <usize>`：网格
- `--body-weight-kg <f64>`：体重

示例：

- `cargo run -p tvp_hrt_cli -- curve --compound estradiol --route injection --dose 5 --t0-h 0 --total-h 240 --steps 800`

### 4.3 simulate（events 驱动）

用途：直接读取 events JSON（DoseEvent 数组）并运行 Oyama-compatible 模拟。

常用参数：

- `--events <path>`：必填
- `--body-weight-kg <f64>`：体重
- `--substance/--compound/--route`：可选，仅用于 envelope 的 `events.meta` 记录（不强行推断 subject/route）

events 文件格式：请参考 `tvp_hrt_engine::model::DoseEvent` 的 JSON 表达（字段例如 `id, route, timeH, doseMG, ester, extras`）。

## 5) 测试

### tvp_cli：effect curve（Journal 风格时间线）

- 输入：`--substance` + `--route` + `--dose`，以及可选 `--t0-h` / `--t1-h`（time-range ingestion）
- 输出：`result` 包含 `timeH`, `level`, `levelRaw`, `auc`, `aucRaw`, `maxRaw`

### tvp_cli：simulate --kind effect|pk

- 读取 `--events <path>` 的 JSON 数组
- `--kind=effect`：使用 L3 durations + strength + convolution（Journal timeline）
- `--kind=pk`：Bateman 单次响应线性叠加

### tvp_hrt_cli：simulate

- 读取 `--events <path>` 的 JSON 数组（DoseEvent）并运行 Oyama-compatible 模拟

## 测试

- HRT 侧已有回归测试：`rust/tvp_hrt_engine/tests/oyama_baseline.rs`
- 运行：在 `rust/` 目录下执行 `cargo test`

