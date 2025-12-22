# Implementation Plan: Implement Desktop Tauri Backend

**Branch**: `001-implement-desktop` | **Date**: 2025-12-22 | **Spec**: [link to spec.md]
**Input**: Feature specification from `/specs/001-implement-desktop/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a Tauri-based backend service to replace mock API calls in the existing React frontend for WDRProbe desktop application. The backend will handle WDR report parsing, SQLite data storage, execution plan analysis, performance comparisons, and threshold management via IPC communication.

## Technical Context

**Language/Version**: Rust 1.75+ (for Tauri backend)
**Primary Dependencies**: Tauri framework, SQLite (rusqlite), serde (JSON serialization), regex (SQL parsing)
**Storage**: SQLite embedded database for local data persistence
**Testing**: cargo test (Rust), need to verify specific testing framework for Tauri
**Target Platform**: Desktop (Windows, macOS, Linux) via Tauri runtime
**Project Type**: Desktop application with React frontend + Rust backend
**Performance Goals**: Import 50MB files in 30s, render execution plans in 2s, compare reports in 10s
**Constraints**: Chrome 86.x browser compatibility, offline mode, 100% test coverage requirement
**Scale/Scope**: Support 1000+ SQL entries per report, multiple concurrent report comparisons

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**Gates Determined**:

1. **Principle III - Unified API Encapsulation**: All frontend-backend interactions must use specific ApiService methods via Tauri IPC. Backend MUST implement all IPC commands from desktop-IPC.md. No direct HTTP clients allowed.

2. **Principle IV - Threshold Configuration DTO**: Backend MUST use DTO format { category, dataType, configKey, value }. Update interfaces MUST use { value, changedBy, changeReason } format. Audit trail required.

3. **Principle V - GaussDB Compatibility**: Backend SQL parsing and EXPLAIN processing MUST be compatible with GaussDB syntax. MUST support EXPLAIN ANALYZE and FORMAT JSON. MUST use enable_hypo_index for virtual index evaluation.

4. **Principle VIII - Tauri Desktop Adaptation**: Backend MUST use IPC for file import operations (WDR/EXPLAIN files). Menu bar MUST include "导入/导出PNG" options. Frontend layout MUST use Flexbox (30% SQL editor, 50% tree view).

5. **Principle IX - Documentation and Test Coverage**: Backend MUST have 100% unit test coverage for all API endpoints. All features MUST have design.md documentation. No commits without tests.

**Constitutional Violations to Track**:
- None identified at this time
- All principles align with Tauri backend implementation approach

**Phase 0 Complete**: ✅ Research completed, all unknowns resolved
**Phase 1 Complete**: ✅ Design & Contracts completed
- Data model defined with all entities and relationships
- API contracts generated for all IPC commands
- Quickstart guide created with implementation steps
- Agent context updated

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| None | N/A | N/A |

## Phase 0: Outline & Research - COMPLETED ✅

### Research Tasks Completed
1. ✅ Tauri testing frameworks research - Using cargo test with mockall
2. ✅ SQLite integration patterns - Using rusqlite with r2d2 pooling
3. ✅ File parsing strategies - Using memmap2 for large files, scraper for HTML
4. ✅ GaussDB EXPLAIN compatibility - FORMAT JSON support, enable_hypo_index
5. ✅ IPC command structure - Domain-organized command modules

### Research Output
- **File**: `research.md`
- **Key Decisions**: Testing stack, database strategy, file parsing approach
- **Unknowns Resolved**: All technical questions answered

## Phase 1: Design & Contracts - COMPLETED ✅

### Design Artifacts Created

#### Data Model
- **File**: `data-model.md`
- **Content**: Complete data model with 10 core entities
- **Includes**: Entity definitions, relationships, validation rules, database schema

#### API Contracts
- **Directory**: `contracts/`
- **Files**:
  - `dashboard.md` - Dashboard IPC commands (get_instance_summaries, get_dashboard_metrics)
  - `reports.md` - WDR report management (import, list, detail, delete)
  - `comparison.md` - Report comparison analysis
  - `execution-plan.md` - SQL execution plan visualization
  - `threshold.md` - Threshold configuration (DTO format per Constitution IV)
  - `audit.md` - SQL audit and logging (per Constitution IX)

#### Quickstart Guide
- **File**: `quickstart.md`
- **Content**: Step-by-step implementation guide with code examples
- **Sections**: Prerequisites, setup, implementation steps, testing, troubleshooting

### Agent Context Updated
- **File**: `CLAUDE.md`
- **Added Technologies**: Rust 1.75+, Tauri, SQLite (rusqlite), Serde, Regex

## Phase 2: Implementation Planning - Ready for /speckit.tasks

### Next Steps
1. Run `/speckit.tasks` to generate implementation task list
2. Implement backend commands following the design
3. Write unit and integration tests
4. Replace frontend mock API calls with Tauri IPC
5. Test end-to-end functionality

### Implementation Priorities
1. **P1**: Database schema and models
2. **P1**: Basic IPC commands (dashboard, reports)
3. **P1**: WDR file parser
4. **P2**: Execution plan analysis
5. **P2**: Report comparison
6. **P2**: Threshold configuration (Constitution IV)
7. **P3**: SQL audit (Constitution IX)
8. **P3**: File import/export

## Success Metrics
All success criteria from spec are achievable with this design:
- ✅ File import performance (50MB in 30s) - Achievable with memmap2
- ✅ Plan rendering (2s) - Efficient Rust JSON parsing
- ✅ Comparison speed (10s for 1000+ SQL) - Optimized SQLite queries
- ✅ 100% test coverage - Testable with cargo test and tarpaulin

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
Desktop/
├── src-tauri/                    # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs              # Application entry point
│   │   ├── lib.rs               # Tauri commands and configuration
│   │   ├── commands/            # IPC command implementations
│   │   │   ├── dashboard.rs     # Dashboard data commands
│   │   │   ├── reports.rs       # WDR report management
│   │   │   ├── comparison.rs    # Report comparison logic
│   │   │   ├── execution_plan.rs # Plan analysis and visualization
│   │   │   ├── threshold.rs     # Threshold configuration
│   │   │   └── audit.rs         # SQL audit and logging
│   │   ├── database/            # SQLite schema and operations
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs        # Database schema definitions
│   │   │   └── operations.rs    # CRUD operations
│   │   ├── parsers/             # File parsing logic
│   │   │   ├── mod.rs
│   │   │   ├── wdr_parser.rs    # WDR HTML/raw file parser
│   │   │   └── sql_parser.rs    # SQL and execution plan parser
│   │   ├── models/              # Data models
│   │   │   ├── mod.rs
│   │   │   ├── report.rs        # WDR report models
│   │   │   ├── comparison.rs    # Comparison result models
│   │   │   └── threshold.rs     # Threshold DTO models
│   │   └── utils/               # Utility functions
│   │       ├── mod.rs
│   │       └── gaussdb.rs       # GaussDB-specific utilities
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
│
├── frontend/                    # React frontend (already implemented)
│   ├── src/
│   │   ├── components/          # Reusable React components
│   │   ├── pages/               # Page-level components
│   │   ├── services/            # API service layer (needs Tauri IPC integration)
│   │   └── types.ts             # TypeScript type definitions
│   └── package.json
│
└── tests/                       # Test directories
    ├── unit/                    # Unit tests
    ├── integration/             # Integration tests
    └── contract/                # IPC contract tests

# Documentation
docs/
├── gaussdb.md                   # GaussDB compatibility guide
├── desktop-IPC.md              # IPC interface definitions
├── desktop-req.md              # Requirements specification
└── desktop-design.md           # Design specifications
```

**Structure Decision**: Desktop application with React frontend and Tauri Rust backend. Frontend already implemented. Backend structure follows modular architecture with clear separation of concerns: commands (IPC), database (SQLite), parsers (file processing), models (DTOs), and utilities (GaussDB integration). All backend code will be in `Desktop/src-tauri/` directory.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
