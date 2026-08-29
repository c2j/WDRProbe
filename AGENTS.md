# WDRProbe — Agent Instructions

## What This Is

WDRProbe is a **Tauri v1 desktop app** for analyzing GaussDB/OpenGauss WDR (Workload Diagnosis Report) files. The frontend (React/TS) is already built; the Rust backend parses HTML WDR reports, stores data in SQLite, and serves it via Tauri IPC commands.

## TDD Workflow (Red → Green → Refactor)

The backend is Rust (Tauri), and the frontend is React/TS. Core testable logic resides on the Rust side (parser, database, commands); the frontend currently lacks a test framework. Before modifying code, confirm whether you are changing the Rust backend, TS frontend, or the Tauri IPC boundary.

### Read Before Modifying
1. Confirm which crate the changes fall into (`crates/wdrprobe-*` or `Desktop/src-tauri`).
2. `wdr_parser_main.rs`, `test_*.rs`, and `cache_io_test/` in the root directory are independent experiments and not part of the App build—TDD gates target `Desktop/src-tauri/`.
3. Run the smallest test relevant to the change first; run the Rust gates before submission.
4. Report according to "Completion Criteria and Reporting" after finishing each cycle.

### Never / Ask first / Always

**Never (Prohibited without exception)**
- Delete, comment out, or skip existing tests: `#[ignore]`, commenting out `#[test]`, or changing assertions to `is_ok()`/`unwrap()`
- Modify assertions in existing human-written tests to accommodate an implementation
- Submit business behaviour without tests first, then "backfill later"
- Write always-passing tests: no assertions, only checking `is_some()`, or only verifying call counts without checking parameters and state
- Use full end-to-end tests to cover changes that could be completed with unit tests
- Submit half-finished work; leave exploratory drafts, temporary scripts, or debugging `dbg!`/`println!` in the main code

**Ask first**
- Modifying existing human-written tests (including assertions and fixtures)
- Adding new runtime dependencies, `unsafe`, new crates, or new external services
- Refactoring untestable code beyond the current change path
- Disabling clippy lints or adding new `#[allow(...)]`

**Always**
- Before modifying legacy paths: write characterization tests first to lock down current observable behaviour
- New behaviour: have a failing behaviour assertion first, then write the minimum implementation
- When hard to test: create a seam first, then write the test
- Test names must describe behaviour: `should_reject_invalid_wdr_html`
- Existing tests fail due to your changes: fix the implementation, do not fix the test (unless explicitly requested by a human)

Test ownership:

| Test Source | Ownership |
|---|---|
| Human-written tests | Read-only |
| New tests for this task | Modifiable until the behaviour is stable |
| Outdated or flaky environmental failures | Report only, do not skip without permission |

### Workflow

**Red** — Write tests before writing production behaviour; they must be collectible and must fail (either assertion failure or compilation failure due to missing APIs are valid Red states). Write characterization tests before modifying existing functionality. Add exactly one behaviour per cycle.

**Green** — Write only the minimum code required to make the current failing test pass. Do not delete or modify failing tests, introduce multiple unverified changes at once, or trade green for broader assertions/`unwrap()`.

**Refactor** — Refactor only after all relevant tests are green; run the same set of tests immediately after refactoring; scope is limited to the current change path.

**Exploration vs Implementation** — Write drafts to verify requirements or solutions if unclear; drafts must not be merged; once the solution is determined, it must be rewritten using TDD.

### Legacy Code and Seams

**Characterization Tests** — Use WDR HTML samples in `example/` as fixtures to lock down existing parser output (must cover both opengauss_v1 and v2 formats).

**Seams (Priority order, later is worse)**
1. trait + generics/`impl Trait`, use fake types for testing (`--features test` provides mockall/rstest)
2. Use types to eliminate illegal states (enum/newtype)
3. Make clocks, IDs, file systems, and DB connections injectable dependencies; use in-memory SQLite / `tempfile` for testing
4. `unsafe` is not a seam. New `unsafe` must be Ask first + `SAFETY` comment

Only add tests to the code paths about to be modified; do not "complete coverage" all at once.

### Test Layering

| Level | Location | What to test |
|---|---|---|
| Unit | `#[cfg(test)] mod tests` inside `src` | parser/model/utility invariants |
| Integration | `Desktop/src-tauri/tests/*.rs` | Command contracts, cross-module behaviour |
| Test-only dependencies | `--features test` | mockall/rstest/criterion fake implementations and parameterization |

Do not stuff content that should test public contracts into `#[cfg(test)]` to read private fields.

### Frontend (TS) Notes

- The frontend currently **has no** test framework. TDD is not mandatory for changes to pure TS logic in `Desktop/frontend/` (e.g., mock fallback in `apiService.ts`, type mapping), but extracting complex pure functions is recommended for future testing; do not claim "tested" without adding tests.
- Changes to Tauri IPC contracts (input parameters/return `Result<T, String>` of `#[tauri::command]`) mean Rust integration tests must change, and frontend `invoke()` wrappers and types must be updated synchronously.

### Rust Never Addendum (every item below is forbidden)
- Use `unwrap`/`expect`/`panic!` for control flow in library code
- Unnecessary `unsafe`; if present, must have `SAFETY` comment
- `cargo update` the entire lockfile at once
- Use `#[allow(...)]` to silence lints that should be fixed

### Commands

```bash
# Unit tests (from Desktop/src-tauri/)
cargo test --test <name>

# Full Rust tests (including mockall/rstest/criterion test dependencies)
cargo test --features test

# Pre-submission gates (see the CI coverage table below for what CI does NOT check)
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo test --features test
```

> Note: Run these commands from `Desktop/src-tauri/` (the App's lib crate) — `pr-checks.yml` uses the same `working-directory`. `cargo test` at the repo root tests the standalone experiment scripts, not the App.

What CI actually covers (`pr-checks.yml`, on `ubuntu-22.04` since PR #4):

| Job | CI runs | CI does NOT run |
|---|---|---|
| `Rust Checks` | `cargo clippy --all-targets -- -D warnings`, `cargo test` | `cargo fmt --all -- --check`, `cargo test --features test` |
| `Frontend Type Check` | `npx tsc --noEmit` | any frontend test — there is no test framework |

> `cargo fmt --all -- --check` and `cargo test --features test` are **local-only gates**. CI will never catch a violation in either, so run them yourself and paste the results in your report.
>
> 🔴 **CI is currently red on `main`.** `crates/wdrprobe-core` has **58 pre-existing clippy violations** under `-D warnings` (`get_first` x10, `redundant_closure` x8, `manual_pattern_char_comparison` x7, `needless_borrow` x5, `manual_strip` x5, and others). These predate CI ever running: `pr-checks.yml` was pinned to the retired `ubuntu-20.04` runner, so every run sat in `queued` until `cancelled` and **not one run completed** between 2026-07-06 and PR #4. Establish the baseline on unmodified `main` first, then separate your own failures from it. Do not add `#[allow]` to silence these (that is Ask first), and do not treat a red baseline as licence to skip the gate.
>
> The frontend has only a type-check job and **no test job** — never claim TS changes are "tested".

### Completion Criteria and Reporting

Before submitting or handing back to a human, confirm:
- [ ] New behaviour has failing → passing tests
- [ ] Modified legacy paths have characterization tests (both v1/v2 WDR formats)
- [ ] Human-written tests have not been deleted, skipped, or rewritten
- [ ] fmt + clippy + test gates have been run
- [ ] No exploratory drafts, root directory experimental scripts, or orphaned artifacts outside `example/` are included

Report for each TDD cycle: 1) What behaviour was tested 2) Which files were changed for the minimum implementation 3) Whether refactoring occurred and boundaries 4) Actual commands and results.

### Quality Judgment (Self-check)
- Will this test fail if the implementation is written incorrectly?
- Am I testing behaviour rather than private implementation details?
- Am I trading green for skips, broader assertions, or unwrap?
- Do the commands come from this file rather than being made up?

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
