# WDRProbe — Agent Instructions

## What This Is

WDRProbe is a **Tauri v1 desktop app** for analyzing GaussDB/OpenGauss WDR (Workload Diagnosis Report) files. The frontend (React/TS) is already built; the Rust backend parses HTML WDR reports, stores data in SQLite, and serves it via Tauri IPC commands.

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
