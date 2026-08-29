# WDRProbe — Agent Instructions

## What This Is

WDRProbe is a **Tauri v1 desktop app** for analyzing GaussDB/OpenGauss WDR (Workload Diagnosis Report) files. The frontend (React/TS) is already built; the Rust backend parses HTML WDR reports, stores data in SQLite, and serves it via Tauri IPC commands.

## TDD 工作流（Red → Green → Refactor）

> This section is written in Chinese to stay identical in wording to the shared TDD policy used across the Heptadecagon repos. The rest of this file remains in English.

本仓库后端是 Rust（Tauri），前端是 React/TS。核心可测逻辑都在 Rust 侧（parser、database、commands）；前端目前无测试框架。改代码前先确认改的是 Rust 后端、TS 前端，还是 Tauri IPC 边界。

### 先读再改
1. 确认改动落在哪个 crate（`crates/wdrprobe-*` 或 `Desktop/src-tauri`）。
2. 根目录的 `wdr_parser_main.rs`、`test_*.rs`、`cache_io_test/` 是独立实验，不是 App 构建的一部分——TDD 门禁针对 `Desktop/src-tauri/`。
3. 先跑与改动相关的最小测试；提交前再跑 Rust 门禁。
4. 完成一个循环后按「完成标准与汇报」汇报。

### Never / Ask first / Always

**Never（不必请示，直接禁止）**
- 删除、注释、跳过已有测试：`#[ignore]`、注释 `#[test]`、断言改成 `is_ok()`/`unwrap()`
- 修改人类已有测试的断言来迁就实现
- 先提交无测试的业务行为，再「回头补」
- 写永真测试：无断言、只检查 `is_some()`、只 verify 调用次数不查参数与状态
- 用全量端到端测试覆盖本可单测完成的改动
- 提交半成品；把探索草稿、临时脚本、调试 `dbg!`/`println!` 留在主代码

**Ask first**
- 改人类已有测试（含断言、fixture）
- 新增运行时依赖、`unsafe`、新 crate、新外部服务
- 为不可测代码做超出当前改动路径的重构
- 关闭 clippy lint、新增 `#[allow]`

**Always**
- 改遗留路径前：先写特征测试，锁定当前可观察行为
- 新行为：先有会失败的行为断言，再写最少实现
- 难以测试时：先造接缝，再写测试
- 测试名描述行为：`should_reject_invalid_wdr_html`
- 现有测试因你的改动失败：修实现，不修测试（除非人类明确要求）

测试权限：

| 测试来源 | 权限 |
|---|---|
| 人类已有测试 | 只读 |
| 本任务新建测试 | 可改，直到该行为稳定 |
| 过时或环境偶发失败 | 只报告，不擅自跳过 |

### 工作流

**Red** — 写生产行为之前先写测试；必须能被收集且必须失败（断言失败或缺失 API 编译失败都算合法 Red）。修改已有功能先写特征测试。一次只加一个行为的测试。

**Green** — 只写让当前失败测试通过的最少代码。禁止删掉/改掉失败测试、一次引入多个未验证变更、用更宽断言/`unwrap()` 换绿。

**Refactor** — 相关测试全绿后才重构；重构后立刻跑同一组测试；范围限于当前改动路径。

**探索 vs 实现** — 需求或方案不清可写草稿验证；草稿不得合并；方案确定后必须走 TDD 重写。

### 遗留代码与接缝

**特征测试** — 用 `example/` 里的 WDR HTML 样例做 fixture，锁定 parser 的现有输出（opengauss_v1 与 v2 两种格式都要覆盖）。

**接缝（优先顺序，靠后的更差）**
1. trait + 泛型/`impl Trait`，测试用假类型（`--features test` 提供 mockall/rstest）
2. 用类型去掉非法状态（enum/newtype）
3. 时钟、ID、文件系统、DB 连接做成可注入依赖；测试用内存 SQLite / tempfile
4. `unsafe` 不是接缝。新增 `unsafe` 必须 Ask first + `SAFETY` 注释

只给即将修改的代码路径补测试，不要一次性「补全覆盖率」。

### 测试分层

| 层级 | 位置 | 测什么 |
|---|---|---|
| 单元 | `src` 内 `#[cfg(test)] mod tests` | parser/模型/工具不变量 |
| 集成 | `Desktop/src-tauri/tests/*.rs` | 命令契约、跨模块行为 |
| 测试专用依赖 | `--features test` | mockall/rstest/criterion 假实现与参数化 |

不要把本该测公共契约的内容塞进 `#[cfg(test)]` 去读私有字段。

### 前端（TS）说明

- 前端目前**没有**测试框架。改动 `Desktop/frontend/` 的纯 TS 逻辑（如 `apiService.ts` 的 mock fallback、类型映射）不强制 TDD，但复杂纯函数建议抽成可导出函数以便后续补测试；不得在未加测试的情况下声称「已测试」。
- Tauri IPC 契约（`#[tauri::command]` 的入参/返回 `Result<T, String>`）改动 = Rust 集成测试要动，前端 `invoke()` 封装与类型也要同步更新。

### Rust Never 补遗
- 库代码用 `unwrap`/`expect`/`panic!` 做控制流
- 无必要 `unsafe`；有则必须 `SAFETY` 注释
- 一次性 `cargo update` 整个 lockfile
- 用 `#[allow(...)]` 静默应修复的 lint

### 命令

```bash
# 单测（从 Desktop/src-tauri/）
cargo test --test <name>

# 全量 Rust 测试（含 mockall/rstest/criterion 测试依赖）
cargo test --features test

# 提交前门禁（本地唯一门禁——CI 跑不起来，见下）
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --features test
```

> 注意：命令在 `Desktop/src-tauri/` 目录执行（App 的 lib crate），`pr-checks.yml` 的 `working-directory` 也是这个。根目录 `cargo test` 测的是独立实验脚本，不是 App。
>
> 🔴 **`pr-checks.yml` 实际上从来没有跑起来过，本地门禁是唯一防线。**
> 两个 job（`Rust Checks` / `Frontend Type Check`）都写死 `runs-on: ubuntu-20.04`，
> 而 GitHub 已下线该 runner 镜像——所有 `PR Checks` run 一律停在 `queued` 直到被 `cancelled`，
> 历史上（可回溯至 2026-07-06 的 `main`）**没有一次 completed**。
> 因此：**不要用「CI 绿」当验收依据**，必须本地把上面 4 条全部跑完并在汇报里贴出结果。
> 修复方向是把 `runs-on` 改成 `ubuntu-22.04`（Tauri v1 依赖 `libwebkit2gtk-4.0-dev`，
> Ubuntu 24.04 已改为 4.1，直接换 `ubuntu-latest` 会因缺包而失败）——属独立的 CI 修复任务。
>
> 前端只有 `Frontend Type Check`（`npm run build` 类型检查），**没有测试 job**——改 TS 不要声称「已测试」。

### 完成标准与汇报

提交或交还人类前，确认：
- [ ] 新行为有失败→通过的测试
- [ ] 修改的遗留路径有特征测试（v1/v2 两种 WDR 格式）
- [ ] 未删除、跳过、改写人类已有测试
- [ ] 已跑 fmt + clippy + test 门禁
- [ ] 没有把探索草稿、根目录实验脚本、`example/` 之外的无主产物带上

每个 TDD 循环汇报：1) 测试了什么行为 2) 最小实现改了哪些文件 3) 是否重构、边界 4) 实际命令与结果。

### 质量判断（自我检查）
- 这条测试在实现写错时会失败吗？
- 我是否在测行为，而不是私有实现细节？
- 我是否用 skip、更宽断言、unwrap 换绿？
- 命令是否来自本文件，而不是我编的？

## Repository Layout

```
Desktop/                 ← The actual Tauri app (all dev work happens here)
  frontend/              ← React + TypeScript + Tailwind (Vite dev server on :1420)
    pages/               ← Route-level components (Dashboard, ReportDetail, etc.)
    components/          ← Shared UI (Layout, UploadDialog, ErrorBoundary, etc.)
    services/apiService.ts  ← Tauri invoke wrappers (still has mock fallbacks)
    types.ts             ← All TypeScript interfaces
  src-tauri/             ← Rust backend
    src/
      main.rs            ← Tauri Builder: DB init, schema setup, command registration
      lib.rs             ← Re-exports all modules
      commands/          ← #[tauri::command] IPC handlers (dashboard, reports, comparison, execution_plan, threshold, audit, export)
      models/            ← Serde-serializable Rust structs (report, comparison, threshold, audit, etc.)
      database/
        schema.rs        ← SQLite DDL + default data seeding
        operations.rs    ← CRUD queries
      parsers/           ← WDR HTML parser, SQL parser (uses scraper + nom)
      utils/             ← error types (WdrProbeError enum), GaussDB helpers, audit utils
      progress/          ← ProgressReporter for long-running ops
    tests/               ← Integration tests (mirrors commands/ structure + integration/ subfolder)
    Cargo.toml           ← Package: wdrprobe-desktop, lib crate
    tauri.conf.json      ← App config, allowlist, CSP disabled, dev port 1420
  package.json           ← Scripts: dev, build, tauri:dev, tauri:build
  vite.config.ts         ← Path aliases: @ → frontend/, @components, @utils, @types

docs/                    ← Design docs (desktop-IPC.md defines all IPC interfaces)
specs/                   ← SpecKit artifacts (spec, plan, data model, contracts)
example/                 ← Sample WDR HTML files (opengauss_v1/v2, test_sql_detail)
```

**Root-level `.rs` files** (`wdr_parser_main.rs`, `test_*.rs`) and `cache_io_test/` are standalone experiments — not part of the Tauri build. The root `Cargo.toml` is a separate `wdr_parser_test` bin crate using `scraper`.

## Build & Run Commands

All commands run from `Desktop/`:

```bash
npm install              # Install frontend deps
npm run tauri:dev        # Dev mode (starts Vite + Tauri, hot reload frontend)
npm run tauri:build      # Production build → src-tauri/target/release/bundle/

# Rust-only (from Desktop/src-tauri/)
cargo test               # Run all tests
cargo test --test <name> # Run specific test file (e.g., --test reports_test)
cargo test --features test  # Include optional test-only deps (mockall, rstest, criterion)
cargo clippy             # Lint

# Frontend-only (from Desktop/)
npm run dev              # Vite dev server without Tauri (port 1420)
npm run build            # TypeScript check + Vite build
```

**Build order**: `npm install` → `npm run tauri:dev` (or `tauri:build`). No separate backend build step — Tauri handles it.

## Key Architecture Facts

- **Tauri v1** (not v2). Uses `tauri::Builder::default()` pattern with `invoke_handler`.
- **Database**: SQLite via `rusqlite` + `r2d2` connection pool. DB file created at `{app_data_dir}/wdrprobe.db` on first launch. Schema auto-initialized in `main.rs::setup()`.
- **IPC**: Frontend calls `invoke("command_name", { args })` → Rust `#[tauri::command]` handlers. All commands registered in `main.rs`. See `docs/desktop-IPC.md` for the full interface spec.
- **Error handling**: Custom `WdrProbeError` enum with `thiserror`. Commands return `Result<T, String>` (Tauri requires String errors).
- **Parsing**: WDR HTML parsed with `scraper` crate. SQL parsed with `nom`. Large files use `memmap2`.
- **Frontend state**: No global state library. Components call `apiService.ts` which wraps `invoke()` calls. Some mock data still present as fallback.
- **Path aliases**: `@` → `frontend/`, `@components` → `frontend/components/`, etc. (configured in `vite.config.ts`).

## Adding a New Tauri Command

1. Add `#[tauri::command]` fn in `Desktop/src-tauri/src/commands/<domain>.rs`
2. Register it in `Desktop/src-tauri/src/main.rs` → `tauri::generate_handler![...]`
3. Add TypeScript wrapper in `Desktop/frontend/services/apiService.ts`
4. Add types to `Desktop/frontend/types.ts` if needed

## Testing

- **Rust tests**: `Desktop/src-tauri/tests/` — organized by domain (reports_test.rs, comparison_test.rs, etc.) plus `integration/` for cross-cutting tests.
- **Test fixtures**: WDR HTML samples in `example/`. Tests reference these for end-to-end parsing validation.
- **Test feature flag**: `--features test` enables `mockall`, `rstest`, `criterion` (optional deps).
- **No frontend tests** currently.

## CI / Release

- GitHub Actions: `.github/workflows/release.yml` — triggered on `v*` tags.
- Builds for: macOS (arm64 + x86_64), Linux x86_64 (ubuntu-20.04 for glibc 2.31 / Kylin OS compat), Linux arm64 (cross-compiled), Windows (x86_64 + arm64).
- Uses `tauri-apps/tauri-action@v0` with `projectPath: Desktop`.
- Rust cache scoped to `Desktop/src-tauri/target`.

## Gotchas

- **CSP is disabled** (`"csp": null` in tauri.conf.json) — intentional for dev, review before production.
- **Two Cargo.toml workspaces**: Root (`wdr_parser_test` bin) and `Desktop/src-tauri/` (`wdrprobe-desktop` lib) are independent crates. Running `cargo test` at root tests the experiment scripts, not the app.
- **Frontend `apiService.ts` still has mock data** — the real Tauri backend is implemented but some mock fallbacks may remain.
- **Chinese-language design docs**: `docs/desktop-IPC.md` and `docs/desktop-design.md` are in Chinese. These are the authoritative IPC interface specs.
- **Linux builds target glibc 2.31** (ubuntu-20.04) for Kylin OS compatibility — don't upgrade the CI runner without considering this.
- **WDR file versions**: Parser handles two HTML formats (opengauss_v1 and v2). Test with both — they have different structures.
- **SQLite bundled**: `rusqlite` uses `bundled` feature — no external SQLite dependency needed.
