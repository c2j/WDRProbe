# Research: Tauri Desktop Backend Implementation

**Date**: 2025-12-22
**Feature**: Implement Desktop Tauri Backend

## Executive Summary

Research conducted on best practices for implementing a Tauri Rust backend for WDRProbe desktop application. Key findings include testing strategies with cargo test and mockall, SQLite integration patterns with rusqlite and connection pooling, and file parsing approaches for large WDR files.

## Decisions Made

### 1. Testing Framework Selection

**Decision**: Use Rust's built-in `cargo test` with `mockall` and `rstest` for unit tests, and a custom test harness for integration tests.

**Rationale**: This is the most mature and widely-adopted testing approach for Tauri applications in 2024-2025. The built-in testing infrastructure is stable and well-integrated with the Rust ecosystem.

**Alternatives Considered**:
- `trybuild` for compile-time testing (rejected: too complex for this use case)
- `insta` for snapshot testing (rejected: not needed for this project)

**Implementation**:
- Unit tests for all Tauri commands using `mockall` for dependency mocking
- Integration tests using a separate test harness that spawns Tauri instances
- Use `tempfile` for isolated test database instances
- Coverage reporting via `cargo-tarpaulin` to achieve 100% coverage requirement

### 2. SQLite Database Strategy

**Decision**: Use `rusqlite` with `r2d2` connection pooling, enable WAL mode, and wrap blocking operations in `spawn_blocking`.

**Rationale**: This provides the best balance of performance, concurrency, and async compatibility. WAL mode is essential for desktop applications with potentially concurrent read/write operations.

**Alternatives Considered**:
- `sqlite3` C library bindings (rejected: more complex, less idiomatic Rust)
- `sqlx` with async SQLite (rejected: adds unnecessary async complexity for desktop use)

**Implementation**:
- Initialize database in app data directory using Tauri `path` API
- Enable WAL mode for better concurrency
- Use `r2d2` connection pool for production, single connection for testing
- Custom error types using `thiserror` for type-safe error handling
- All database operations in `spawn_blocking` to avoid blocking async runtime

### 3. File Parsing Strategy

**Decision**: Use `scraper` for HTML parsing, `memmap2` for large file processing, and streaming parsers with progress reporting via Tauri IPC events.

**Rationale**: WDR files can be large (>50MB) and need efficient parsing. Memory mapping provides the best performance for large files, while streaming prevents memory exhaustion.

**Alternatives Considered**:
- Read entire file into memory (rejected: risk of OOM for large files)
- `tokio-util` codecs (rejected: memmap2 provides better performance for sequential reads)

**Implementation**:
- HTML WDR reports parsed with `scraper` CSS selectors
- Large raw files processed with `memmap2` for memory efficiency
- Streaming approach for files >50MB to prevent memory issues
- Progress updates via `window.emit` to frontend for long-running operations
- Use `mpsc` channels for progress reporting pattern

### 4. WDR File Parsing Approach

**Decision**: Implement custom parser using `nom` parser combinator for proprietary WDR raw format, and `scraper` for HTML format.

**Rationale**: WDR files are database-specific formats that require precise parsing. Using parser combinators gives us full control over the parsing logic and error handling.

**Implementation**:
- HTML format: Parse with `scraper` to extract SQL statistics, efficiency metrics, and object statistics
- Raw format: Define parser grammar using `nom` combinators
- Extract key entities: SQL text, execution metrics (CPU time, IO, row counts), wait events, object statistics
- Store parsed data in SQLite with appropriate schema
- Handle malformed files gracefully with descriptive error messages

### 5. GaussDB EXPLAIN Plan Integration

**Decision**: Implement EXPLAIN plan parser compatible with GaussDB FORMAT JSON output, with support for enable_hypo_index simulation.

**Rationale**: Constitution requires GaussDB compatibility. The parser must handle GaussDB-specific plan format and support virtual index evaluation.

**Implementation**:
- Parse JSON format EXPLAIN output from GaussDB
- Convert plan tree to structured data for visualization
- Support virtual index simulation via enable_hypo_index parameter
- Highlight problematic nodes (sequential scans, high cost operations)
- Generate optimization suggestions based on plan analysis

### 6. IPC Command Structure

**Decision**: Organize commands by domain (dashboard, reports, comparison, execution_plan, threshold, audit) with separate modules for each.

**Rationale**: This provides clear separation of concerns and makes the codebase maintainable. Each command module focuses on a specific domain.

**Implementation**:
- `dashboard.rs`: get_instance_summaries, get_dashboard_metrics
- `reports.rs`: get_wdr_reports, get_wdr_report_detail, delete_wdr_report
- `comparison.rs`: get_comparisons, get_comparison_summary, get_comparison_details
- `execution_plan.rs`: get_wdr_hot_sqls, get_execution_plan
- `threshold.rs`: get_threshold_configs, update_threshold, batch_update_thresholds
- `audit.rs`: get_sql_audit_issues, get_audit_logs

All commands use `#[tauri::command]` attribute and return `Result<T, String>` for error handling.

### 7. Data Models and DTOs

**Decision**: Define strongly-typed Rust structs for all data models, use Serde for serialization/deserialization, and implement DTO format for threshold configuration per Constitution.

**Rationale**: Type safety is critical for a desktop application handling important database analysis data. Serde provides automatic JSON serialization for IPC communication.

**Implementation**:
- Models in `models/` directory with one file per domain
- Use `serde` and `serde_json` for all serialization
- Implement DTO format for thresholds: { category, dataType, configKey, value }
- Update format: { value, changedBy, changeReason }
- Maintain audit trail for all threshold changes

## Key Findings

### Performance Targets Achievability

All success criteria performance targets are achievable with the recommended approach:
- 50MB file import in 30s: Achievable with memmap2 and streaming parsing
- Execution plan rendering in 2s: JSON parsing and tree construction is fast in Rust
- Report comparison in 10s: SQLite can handle 1000+ SQL entries efficiently with proper indexing
- 100% test coverage: Testable with cargo test and tarpaulin coverage tool

### Constitutional Compliance

The implementation approach satisfies all constitutional requirements:
- ✅ Principle III (Unified API): All commands via Tauri IPC, no direct HTTP
- ✅ Principle IV (Threshold DTO): DTO format with audit trail
- ✅ Principle V (GaussDB): EXPLAIN FORMAT JSON support, enable_hypo_index
- ✅ Principle VIII (Tauri Adaptation): IPC for file operations
- ✅ Principle IX (Test Coverage): 100% coverage requirement addressed

### Technical Risks and Mitigations

**Risk**: Large WDR files causing memory issues
**Mitigation**: Use memmap2 and streaming parsing to avoid loading entire files

**Risk**: Database corruption
**Mitigation**: Use WAL mode, proper error handling, and transaction rollback

**Risk**: Frontend-backend IPC failures
**Mitigation**: Comprehensive integration tests, proper error propagation

**Risk**: GaussDB compatibility issues
**Mitigation**: Extensive testing with real GaussDB EXPLAIN output, fallback parsing

## Unknowns Resolved

### What testing framework to use?
**Answer**: Rust built-in cargo test with mockall for mocking, rstest for parameterized tests. Integration tests via custom test harness.

### How to structure SQLite in Tauri?
**Answer**: rusqlite with r2d2 pooling, WAL mode, spawn_blocking for async compatibility, custom error types.

### How to parse large WDR files efficiently?
**Answer**: memmap2 for large files, scraper for HTML, nom for proprietary format, streaming with progress reporting.

### How to achieve 100% test coverage?
**Answer**: cargo test for unit tests, mockall for mocking, tarpaulin for coverage reporting, integration tests for IPC.

## Next Steps

1. Proceed to Phase 1: Design & Contracts
2. Create data model definitions
3. Generate API contracts from functional requirements
4. Update agent context files
5. Create quickstart documentation

## References

- Tauri v1/v2 documentation on commands and IPC
- rusqlite documentation for database operations
- memmap2 crate for large file processing
- scraper crate for HTML parsing
- nom crate for parser combinators
- cargo-tarpaulin for test coverage
- GaussDB EXPLAIN documentation in gaussdb.md
