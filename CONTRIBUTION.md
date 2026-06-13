# Contributing to WDRProbe

Thank you for your interest in contributing to WDRProbe! This document covers everything you need to get started as a developer.

---

## Table of Contents

1. [Development Environment](#development-environment)
2. [Project Structure](#project-structure)
3. [Running the Project](#running-the-project)
4. [Coding Conventions](#coding-conventions)
5. [Adding a New Tauri Command](#adding-a-new-tauri-command)
6. [Adding a New Frontend Page](#adding-a-new-frontend-page)
7. [Testing](#testing)
8. [Code Quality](#code-quality)
9. [Pull Request Process](#pull-request-process)
10. [CI/CD Pipeline](#cicd-pipeline)
11. [Release Process](#release-process)

---

## Development Environment

### Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| [Node.js](https://nodejs.org/) | 18+ (LTS) | Frontend build |
| [Rust](https://www.rust-lang.org/) | stable | Backend (edition 2021) |
| [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites) | v1 | System dependencies required per platform |
| Git | latest | Version control |

### Platform-Specific System Dependencies

#### macOS

```bash
# Xcode command line tools
xcode-select --install
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.0-dev \
  libgtk-3-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf
```

> **Important**: Tauri v1 requires `webkit2gtk-4.0` (not 4.1). Do not upgrade this dependency.

#### Windows

Install [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/).

### Initial Setup

```bash
git clone <repository-url>
cd WDRProbe/Desktop
npm install
```

The `npm install` step also triggers Rust compilation when you first run `npm run tauri:dev`.

---

## Project Structure

```
WDRProbe/
├── Desktop/                         # ← ALL development happens here
│   ├── frontend/                    # React + TypeScript frontend
│   │   ├── pages/                   # Route-level components (11 pages)
│   │   ├── components/              # Shared UI (Layout, UploadDialog, etc.)
│   │   ├── context/                 # React Context providers (I18n, Plan, WDR)
│   │   ├── services/
│   │   │   └── apiService.ts        # Tauri IPC invoke wrappers + mock fallbacks
│   │   ├── utils/                   # Frontend utility functions
│   │   ├── types.ts                 # All TypeScript interfaces
│   │   ├── App.tsx                  # Main routing (HashRouter)
│   │   ├── index.tsx                # React entry point
│   │   └── index.css                # Global styles + Tailwind directives
│   │
│   ├── src-tauri/                   # Rust backend
│   │   ├── src/
│   │   │   ├── main.rs              # Tauri Builder: setup + command registration
│   │   │   ├── lib.rs               # Library exports
│   │   │   ├── commands/            # #[tauri::command] IPC handlers
│   │   │   │   ├── dashboard.rs
│   │   │   │   ├── reports.rs
│   │   │   │   ├── execution_plan.rs
│   │   │   │   ├── comparison.rs
│   │   │   │   ├── threshold.rs
│   │   │   │   ├── audit.rs
│   │   │   │   └── export.rs
│   │   │   ├── database/
│   │   │   │   ├── mod.rs           # Connection pool, type definitions
│   │   │   │   ├── schema.rs        # DDL + default data seeding
│   │   │   │   └── operations.rs    # DatabaseOperations trait + impl
│   │   │   ├── parsers/
│   │   │   │   ├── wdr_parser.rs            # HTML WDR parser (scraper)
│   │   │   │   ├── complete_wdr_parser.rs   # Full WDR parser (all sections)
│   │   │   │   └── sql_parser.rs            # SQL execution plan parser (nom)
│   │   │   ├── models/              # Serde-serializable domain structs
│   │   │   ├── utils/               # Error types, audit, GaussDB helpers
│   │   │   └── progress/            # ProgressReporter for long ops
│   │   ├── tests/                   # Integration tests (see Testing section)
│   │   ├── Cargo.toml
│   │   └── tauri.conf.json
│   │
│   ├── package.json                 # npm scripts
│   └── vite.config.ts               # Vite config with path aliases
│
├── docs/                            # Design docs (Chinese)
├── specs/                           # SpecKit artifacts
├── example/                         # Sample WDR HTML files for testing
├── .github/workflows/               # CI/CD
└── AGENTS.md                        # AI development assistant instructions
```

> **Note**: Root-level `.rs` files (`wdr_parser_main.rs`, `test_*.rs`) and `cache_io_test/` are standalone experiments — NOT part of the Tauri build. The root `Cargo.toml` is a separate crate.

### Path Aliases (vite.config.ts)

| Alias | Path |
|-------|------|
| `@` | `./frontend` |
| `@components` | `./frontend/components` |
| `@utils` | `./frontend/utils` |
| `@types` | `./frontend/types` |

---

## Running the Project

### Development Mode

```bash
cd Desktop
npm run tauri:dev
```

This starts Vite dev server on `http://localhost:1420` and launches the Tauri app window with hot-reload.

### Frontend-Only Mode

```bash
cd Desktop
npm run dev
```

Runs Vite without Tauri. Useful for pure UI development. Note: Tauri IPC calls will fall back to mock data in `apiService.ts`.

### Production Build

```bash
cd Desktop
npm run tauri:build
```

Build artifacts: `Desktop/src-tauri/target/release/bundle/`

---

## Coding Conventions

### Rust

- **Edition**: 2021
- **Error handling**: Use `WdrProbeError` enum (defined in `utils/error.rs`). All Tauri commands return `Result<T, String>` (Tauri requires String errors).
- **Naming**: `snake_case` for functions/variables, `PascalCase` for types/structs
- **Serialization**: All models derive `Serialize`, `Deserialize` from `serde`. Use `#[serde(rename_all = "camelCase")]` for Tauri IPC compatibility.
- **Database**: Use `DatabaseOperations` trait methods. Never write raw SQL outside of `operations.rs`.
- **No `unwrap()` in production code** — use proper error propagation with `?` operator.
- **Audit logging**: All mutating operations (threshold updates, imports, exports) must log to `audit_logs` table.

### TypeScript / React

- **Strict mode**: TypeScript strict mode is enabled
- **No `any` type**: Never use `as any` or `@ts-ignore`. Define proper types in `types.ts`.
- **Functional components**: Use function components with hooks (no class components)
- **Naming**: `PascalCase` for components/types, `camelCase` for functions/variables
- **Styling**: TailwindCSS utility classes directly in JSX. No CSS modules.
- **i18n**: All user-facing strings must use `t('key')` from `useI18n()` hook. Add both EN and ZH translations in `I18nContext.tsx`.
- **State**: React Context for global state (I18n, Plan, WDR). No external state library.

### File Organization

- One React component per file
- Page components go in `frontend/pages/`
- Shared components go in `frontend/components/`
- New types go in `frontend/types.ts`
- New IPC wrappers go in `frontend/services/apiService.ts`

---

## Adding a New Tauri Command

This is the most common contribution pattern. Follow these steps:

### Step 1: Define the Model (if needed)

Add Rust structs in `src-tauri/src/models/`:

```rust
// src-tauri/src/models/my_feature.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyFeature {
    pub id: i64,
    pub name: String,
    pub value: f64,
}
```

Register the module in `src-tauri/src/models/mod.rs`:

```rust
pub mod my_feature;
pub use my_feature::*;
```

### Step 2: Add Database Operations (if needed)

Add methods to the `DatabaseOperations` trait in `src-tauri/src/database/operations.rs`:

```rust
fn create_my_feature(&self, feature: &MyFeature) -> Result<i64>;
fn get_my_feature(&self, id: i64) -> Result<Option<MyFeature>>;
fn list_my_features(&self) -> Result<Vec<MyFeature>>;
```

Implement the methods for `DatabasePool` in the same file.

### Step 3: Write the Command

Create or add to a file in `src-tauri/src/commands/`:

```rust
// src-tauri/src/commands/my_feature.rs
use tauri::State;
use crate::database::DatabasePool;
use crate::models::MyFeature;

#[tauri::command]
pub async fn get_my_feature(
    pool: State<'_, DatabasePool>,
    id: i64,
) -> Result<MyFeature, String> {
    let conn = pool.get().map_err(|e| e.to_string())?;
    // Use DatabaseOperations trait
    let result = conn.get_my_feature(id)
        .map_err(|e| e.to_string())?;
    result.ok_or_else(|| "Feature not found".to_string())
}
```

### Step 4: Register the Command

In `src-tauri/src/main.rs`, add to the `generate_handler!` macro:

```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    my_feature::get_my_feature,
])
```

### Step 5: Add TypeScript Types

In `frontend/types.ts`:

```typescript
export interface MyFeature {
  id: number;
  name: string;
  value: number;
}
```

### Step 6: Add API Service Wrapper

In `frontend/services/apiService.ts`:

```typescript
export const ApiService = {
  // ... existing services ...

  getMyFeature: async (id: number): Promise<MyFeature> => {
    if (isTauri()) return invoke('get_my_feature', { id });
    return Promise.resolve(MOCK_MY_FEATURE); // optional mock
  },
};
```

### Step 7: Use in a React Component

```typescript
import { ApiService } from '../services/apiService';
import { MyFeature } from '../types';

const [feature, setFeature] = useState<MyFeature | null>(null);

useEffect(() => {
  ApiService.getMyFeature(1).then(setFeature);
}, []);
```

---

## Adding a New Frontend Page

### Step 1: Create the Page Component

```typescript
// frontend/pages/MyPage.tsx
import React from 'react';

const MyPage: React.FC = () => {
  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold">My Page</h1>
    </div>
  );
};

export default MyPage;
```

### Step 2: Add Route

In `frontend/App.tsx`:

```typescript
import MyPage from "./pages/MyPage";

// In <Routes>:
<Route path="/my-page" element={<MyPage />} />
```

### Step 3: Add Menu Item (optional)

In `frontend/components/Layout.tsx`, add to `MENU_ITEMS`:

```typescript
const MENU_ITEMS = [
  // ... existing items ...
  { path: '/my-page', labelKey: 'menu.myPage', icon: MyIcon },
];
```

### Step 4: Add i18n Keys

In `frontend/context/I18nContext.tsx`, add translations:

```typescript
// English
'menu.myPage': 'My Page',

// Chinese
'menu.myPage': '我的页面',
```

---

## Testing

### Test Organization

Tests are located in `Desktop/src-tauri/tests/`:

```
tests/
├── dashboard_test.rs              # Dashboard commands
├── reports_test.rs                # Report CRUD
├── comparison_test.rs             # Comparison logic
├── threshold_test.rs              # Threshold management
├── audit_test.rs                  # Audit functionality
├── execution_plan_test.rs         # Execution plan tests
├── export_test.rs                 # Export/import
├── sql_analysis_test.rs           # SQL analysis
├── end_to_end_test.rs             # E2E workflow tests
├── test_real_wdr_file.rs          # Real WDR file parsing
├── test_complete_wdr_parser.rs    # Complete parser validation
├── test_wdr_parsing.rs            # WDR parsing tests
├── test_comparison_algorithm.rs   # Comparison algorithm
├── parse_real_wdr_test.rs         # Real WDR parsing
├── integration/                   # Integration tests
│   ├── test_export_import.rs
│   ├── test_audit.rs
│   ├── test_threshold_audit.rs
│   ├── test_comparison.rs
│   └── test_execution_plan.rs
└── audit/
    └── test_detection_rules.rs    # Audit detection rules
```

### Running Tests

```bash
# All Rust tests
cd Desktop/src-tauri
cargo test

# Specific test file
cargo test --test reports_test

# With test-only features (mockall, rstest, criterion, tempfile)
cargo test --features test

# Run a specific test
cargo test --test reports_test -- test_name

# Lint
cargo clippy
```

### Writing Tests

#### Test Patterns

Each test file uses `tempfile::TempDir` for database isolation:

```rust
use tempfile::TempDir;
use wdrprobe_desktop_lib::database::{init_database, initialize_schema, DatabaseOperations};

fn setup_test_db() -> (TempDir, DatabasePool) {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let db_path = tmp.path().join("test.db");
    let pool = init_database(db_path.to_str().unwrap()).expect("Failed to init DB");
    let conn = pool.get().expect("Failed to get connection");
    initialize_schema(&conn).expect("Failed to init schema");
    (tmp, pool)
}

#[tokio::test]
async fn test_create_and_get_report() {
    let (_tmp, pool) = setup_test_db();
    let conn = pool.get().unwrap();

    // Create
    let report = WdrReport { /* ... */ };
    let id = conn.create_wdr_report(&report).unwrap();

    // Verify
    let result = conn.get_wdr_report(id).unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().instance_name, report.instance_name);
}
```

#### Test Fixtures

Sample WDR HTML files are in `example/`:
- `opengauss_v1.html` — v1 format
- `opengauss_v2.html` — v2 format
- `test_sql_detail_wdr.html` — SQL detail format

Use these for parser tests:

```rust
#[test]
fn test_parse_real_v1_report() {
    let html = std::fs::read_to_string("../../example/opengauss_v1.html").unwrap();
    let report = parse_complete_wdr_report(&html, "test-instance").unwrap();
    assert!(!report.top_sql.is_empty());
}
```

### Frontend Tests

> **Note**: No frontend tests currently exist. If adding frontend tests, consider using Vitest + React Testing Library.

---

## Code Quality

### Before Submitting a PR

1. **Rust linting**:
   ```bash
   cd Desktop/src-tauri
   cargo clippy -- -D warnings
   ```

2. **TypeScript check**:
   ```bash
   cd Desktop
   npm run build  # runs tsc + vite build
   ```

3. **Tests pass**:
   ```bash
   cd Desktop/src-tauri
   cargo test
   ```

### What NOT to Do

- **No `as any`, `@ts-ignore`, or `@ts-expect-error`** in TypeScript
- **No `unwrap()` in production Rust code** (only in tests)
- **No raw SQL outside `operations.rs`**
- **No hardcoded user-facing strings** — use i18n
- **No CSS modules** — use TailwindCSS utility classes
- **Do not commit** `node_modules/`, `target/`, `dist/`, or `wdrprobe.db`

---

## Pull Request Process

1. **Fork** the repository and create a feature branch:
   ```bash
   git checkout -b feature/my-new-feature
   ```

2. **Write code** following the conventions above.

3. **Write or update tests** for your changes.

4. **Verify locally**:
   ```bash
   # Rust
   cd Desktop/src-tauri
   cargo test && cargo clippy -- -D warnings

   # Frontend
   cd Desktop
   npm run build
   ```

5. **Commit** with clear messages:
   ```bash
   git commit -m "feat: add SQL pattern detection for unused indexes"
   git commit -m "fix: handle empty Top SQL array in WDR parser"
   git commit -m "docs: update threshold configuration guide"
   ```

   Commit message prefixes: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`

6. **Push and create a Pull Request** with:
   - Clear title and description
   - Reference to any related issues
   - Screenshots for UI changes

7. **Address review feedback** — push additional commits to the same branch.

---

## CI/CD Pipeline

The release pipeline is defined in `.github/workflows/release.yml`.

### Trigger

Push a tag matching `v*`:
```bash
git tag v0.2.0
git push origin v0.2.0
```

### Build Matrix

| Platform | Target | Notes |
|----------|--------|-------|
| macOS | `aarch64-apple-darwin` | Apple Silicon |
| macOS | `x86_64-apple-darwin` | Intel |
| Linux | `x86_64-unknown-linux-gnu` | Ubuntu 20.04 (glibc 2.31 for Kylin OS) |
| Linux | `aarch64-unknown-linux-gnu` | Cross-compiled |
| Windows | `x86_64-pc-windows-msvc` | |
| Windows | `aarch64-pc-windows-msvc` | |

### Key CI Details

- **Linux uses Ubuntu 20.04** for glibc 2.31 compatibility with Kylin OS — do not change the runner version without considering this.
- **Linux arm64 is cross-compiled** using Focal arm64 ports with `gcc-aarch64-linux-gnu`.
- **Tauri v1** uses `webkit2gtk-4.0` (not 4.1).
- **Rust cache** is scoped to `Desktop/src-tauri/target` to avoid conflicts with root-level experiment crates.
- **Concurrency groups** prevent parallel builds for the same tag.

---

## Release Process

1. Update version in all three locations:
   - `Desktop/package.json` → `"version"`
   - `Desktop/src-tauri/Cargo.toml` → `version`
   - `Desktop/src-tauri/tauri.conf.json` → `"version"`

2. Update `CHANGELOG` or release notes if applicable.

3. Commit and tag:
   ```bash
   git commit -am "chore: bump version to 0.2.0"
   git tag v0.2.0
   git push origin main --tags
   ```

4. CI will build all 6 platform targets and create a **draft GitHub Release**.

5. Review the draft release, add release notes, and publish.

---

## Questions?

- Open a [GitHub Issue](../../issues) for bugs or feature requests.
- Read [docs/DeveloperGuide.md](docs/DeveloperGuide.md) for architecture details.
- Read [docs/desktop-IPC.md](docs/desktop-IPC.md) for the authoritative IPC interface specification.

Thank you for contributing!
