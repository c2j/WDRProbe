# Tasks: Implement Desktop Tauri Backend

**Input**: Design documents from `/specs/001-implement-desktop/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Include test tasks to achieve 100% coverage per Constitution Principle IX

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create Rust module structure per implementation plan in Desktop/src-tauri/src/
- [ ] T002 Update Cargo.toml with Tauri, SQLite, and parsing dependencies
- [ ] T003 [P] Setup Rust formatting (rustfmt) and linting (clippy) configuration
- [ ] T004 [P] Create empty module files for commands/, database/, parsers/, models/, utils/
- [ ] T005 [P] Initialize Tauri application structure in lib.rs and main.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T006 Implement database module with connection pooling in Desktop/src-tauri/src/database/mod.rs
- [ ] T007 Create SQLite schema initialization in Desktop/src-tauri/src/database/schema.rs
- [ ] T008 [P] Define core data models (WdrReport, TopSql, ThresholdConfig) in Desktop/src-tauri/src/models/mod.rs
- [ ] T009 [P] Define comparison models (WdrComparison, SqlComparisonMetric) in Desktop/src-tauri/src/models/comparison.rs
- [ ] T010 [P] Define execution plan models (ExecutionPlanNode, SqlExecutionPlan) in Desktop/src-tauri/src/models/execution_plan.rs
- [ ] T011 [P] Define audit models (SqlAuditIssue, AuditLog) in Desktop/src-tauri/src/models/audit.rs
- [ ] T012 Setup error handling infrastructure with custom error types
- [ ] T013 Create audit logging utilities per Constitution Principle IX
- [ ] T014 Configure database operations module with CRUD operations

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Launch Desktop Application (Priority: P1) 🎯 MVP

**Goal**: Users start WDRProbe and see dashboard with real data from backend

**Independent Test**: Launch desktop app and verify dashboard displays instance summaries and metrics without errors

### Tests for User Story 1

- [ ] T015 [P] [US1] Unit test for get_instance_summaries in Desktop/src-tauri/tests/dashboard_test.rs
- [ ] T016 [P] [US1] Unit test for get_dashboard_metrics in Desktop/src-tauri/tests/dashboard_test.rs
- [ ] T017 [P] [US1] Integration test for dashboard data loading in Desktop/src-tauri/tests/integration/test_dashboard.rs

### Implementation for User Story 1

- [ ] T018 [P] [US1] Implement dashboard models (InstanceSummary, DashboardMetrics) in Desktop/src-tauri/src/models/dashboard.rs
- [ ] T019 [US1] Implement get_instance_summaries command in Desktop/src-tauri/src/commands/dashboard.rs
- [ ] T020 [US1] Implement get_dashboard_metrics command in Desktop/src-tauri/src/commands/dashboard.rs
- [ ] T021 [US1] Implement database operations for dashboard in Desktop/src-tauri/src/database/operations.rs
- [ ] T022 [US1] Register dashboard commands in lib.rs
- [ ] T023 [US1] Update frontend API service to call Tauri IPC in Desktop/frontend/src/services/api.ts
- [ ] T024 [US1] Add error handling and validation for dashboard commands

**Checkpoint**: At this point, User Story 1 should be fully functional - dashboard loads with backend data

---

## Phase 4: User Story 2 - Import and Manage WDR Reports (Priority: P1)

**Goal**: Users import WDR files, view reports list, and manage reports

**Independent Test**: Import a WDR file, verify it appears in list, view details, and delete it

### Tests for User Story 2

- [ ] T025 [P] [US2] Unit test for get_wdr_reports in Desktop/src-tauri/tests/reports_test.rs
- [ ] T026 [P] [US2] Unit test for import_wdr_report in Desktop/src-tauri/tests/reports_test.rs
- [ ] T027 [P] [US2] Integration test for WDR file import flow in Desktop/src-tauri/tests/integration/test_report_import.rs
- [ ] T028 [P] [US2] Test for WDR HTML parser in Desktop/src-tauri/tests/parsers/test_wdr_parser.rs

### Implementation for User Story 2

- [ ] T029 [P] [US2] Implement WDR HTML parser in Desktop/src-tauri/src/parsers/wdr_parser.rs
- [ ] T030 [P] [US2] Implement WDR raw file parser in Desktop/src-tauri/src/parsers/wdr_parser.rs
- [ ] T031 [US2] Implement import_wdr_report command with progress tracking
- [ ] T032 [US2] Implement get_wdr_reports command in Desktop/src-tauri/src/commands/reports.rs
- [ ] T033 [US2] Implement get_wdr_report_detail command in Desktop/src-tauri/src/commands/reports.rs
- [ ] T034 [US2] Implement delete_wdr_report command with audit logging
- [ ] T035 [US2] Implement database operations for reports in Desktop/src-tauri/src/database/operations.rs
- [ ] T036 [US2] Register report commands in lib.rs
- [ ] T037 [US2] Update frontend services to call report IPC commands
- [ ] T038 [US2] Add file validation and error handling

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Analyze SQL Execution Plans (Priority: P1)

**Goal**: Users view execution plan visualizations with hot SQL queries

**Independent Test**: Click hot SQL from report and verify plan tree renders with cost info

### Tests for User Story 3

- [ ] T039 [P] [US3] Unit test for get_execution_plan in Desktop/src-tauri/tests/execution_plan_test.rs
- [ ] T040 [P] [US3] Unit test for analyze_execution_plan in Desktop/src-tauri/tests/execution_plan_test.rs
- [ ] T041 [P] [US3] Test for SQL parser in Desktop/src-tauri/tests/parsers/test_sql_parser.rs
- [ ] T042 [P] [US3] Integration test for plan visualization flow in Desktop/src-tauri/tests/integration/test_execution_plan.rs

### Implementation for User Story 3

- [ ] T043 [P] [US3] Implement SQL parser in Desktop/src-tauri/src/parsers/sql_parser.rs
- [ ] T044 [P] [US3] Implement execution plan parser (GaussDB FORMAT JSON compatible)
- [ ] T045 [US3] Implement get_wdr_hot_sqls command in Desktop/src-tauri/src/commands/execution_plan.rs
- [ ] T046 [US3] Implement get_execution_plan command in Desktop/src-tauri/src/commands/execution_plan.rs
- [ ] T047 [US3] Implement analyze_execution_plan command with optimization suggestions
- [ ] T048 [US3] Implement GaussDB compatibility utilities in Desktop/src-tauri/src/utils/gaussdb.rs
- [ ] T049 [US3] Implement enable_hypo_index simulation per Constitution V
- [ ] T050 [US3] Register execution plan commands in lib.rs
- [ ] T051 [US3] Update frontend to integrate with plan visualizer
- [ ] T052 [US3] Add async loading for complex plan trees per Constitution VII

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently

---

## Phase 6: User Story 4 - Compare WDR Reports (Priority: P2)

**Goal**: Users compare two reports to identify performance changes

**Independent Test**: Select two reports, run comparison, view results with metric changes

### Tests for User Story 4

- [ ] T053 [P] [US4] Unit test for create_comparison in Desktop/src-tauri/tests/comparison_test.rs
- [ ] T054 [P] [US4] Unit test for get_comparison_summary in Desktop/src-tauri/tests/comparison_test.rs
- [ ] T055 [P] [US4] Integration test for report comparison flow in Desktop/src-tauri/tests/integration/test_comparison.rs
- [ ] T056 [P] [US4] Test for comparison algorithm in Desktop/src-tauri/tests/test_comparison_algorithm.rs

### Implementation for User Story 4

- [ ] T057 [US4] Implement comparison algorithm in Desktop/src-tauri/src/commands/comparison.rs
- [ ] T058 [US4] Implement create_comparison command with performance score calculation
- [ ] T059 [US4] Implement get_comparison_summary command
- [ ] T060 [US4] Implement get_comparison_details command
- [ ] T061 [US4] Implement delete_comparison command
- [ ] T062 [US4] Implement database operations for comparisons
- [ ] T063 [US4] Register comparison commands in lib.rs
- [ ] T064 [US4] Update frontend to integrate comparison UI
- [ ] T065 [US4] Add key findings generation algorithm

**Checkpoint**: User Story 4 complete - reports can be compared

---

## Phase 7: User Story 5 - Configure Performance Thresholds (Priority: P2)

**Goal**: Users adjust threshold values with DTO format and audit trail

**Independent Test**: Update threshold and verify it affects highlighting in reports

### Tests for User Story 5

- [ ] T066 [P] [US5] Unit test for update_threshold in Desktop/src-tauri/tests/threshold_test.rs
- [ ] T067 [P] [US5] Unit test for batch_update_thresholds in Desktop/src-tauri/tests/threshold_test.rs
- [ ] T068 [P] [US5] Test threshold DTO validation per Constitution IV
- [ ] T069 [P] [US5] Integration test for threshold audit logging per Constitution IX

### Implementation for User Story 5

- [ ] T070 [US5] Implement threshold DTO models per Constitution IV in Desktop/src-tauri/src/models/threshold.rs
- [ ] T071 [US5] Implement get_threshold_configs command in Desktop/src-tauri/src/commands/threshold.rs
- [ ] T072 [US5] Implement update_threshold command with DTO validation
- [ ] T073 [US5] Implement batch_update_thresholds command
- [ ] T074 [US5] Implement apply_threshold_template command
- [ ] T075 [US5] Implement threshold audit logging per Constitution IX
- [ ] T076 [US5] Create default threshold templates
- [ ] T077 [US5] Register threshold commands in lib.rs
- [ ] T078 [US5] Update frontend threshold configuration UI
- [ ] T079 [US5] Add threshold validation and error handling

**Checkpoint**: User Story 5 complete - thresholds configurable with audit trail

---

## Phase 8: User Story 6 - View SQL Audit Results (Priority: P3)

**Goal**: Users review detected SQL issues with recommendations

**Independent Test**: View SQL audit page and see detected issues with severity levels

### Tests for User Story 6

- [ ] T080 [P] [US6] Unit test for run_sql_audit in Desktop/src-tauri/tests/audit_test.rs
- [ ] T081 [P] [US6] Unit test for get_sql_audit_issues in Desktop/src-tauri/tests/audit_test.rs
- [ ] T082 [P] [US6] Test SQL audit detection rules in Desktop/src-tauri/tests/audit/test_detection_rules.rs
- [ ] T083 [P] [US6] Integration test for audit workflow in Desktop/src-tauri/tests/integration/test_audit.rs

### Implementation for User Story 6

- [ ] T084 [US6] Implement SQL audit detection rules in Desktop/src-tauri/src/commands/audit.rs
- [ ] T085 [US6] Implement run_sql_audit command to scan reports
- [ ] T086 [US6] Implement get_sql_audit_issues command
- [ ] T087 [US6] Implement update_audit_issue_status command
- [ ] T088 [US6] Implement bulk_update_audit_issues command
- [ ] T089 [US6] Implement database operations for audit
- [ ] T090 [US6] Register audit commands in lib.rs
- [ ] T091 [US6] Update frontend SQL audit page
- [ ] T092 [US6] Add optimization recommendations generator

**Checkpoint**: User Story 6 complete - SQL audit functional

---

## Phase 9: User Story 7 - Export and Import Data (Priority: P3)

**Goal**: Users export/import reports and configurations

**Independent Test**: Export report and import into fresh instance with 100% data fidelity

### Tests for User Story 7

- [ ] T093 [P] [US7] Unit test for export_wdr_report in Desktop/src-tauri/tests/export_test.rs
- [ ] T094 [P] [US7] Unit test for data import validation in Desktop/src-tauri/tests/export_test.rs
- [ ] T095 [P] [US7] Integration test for export/import cycle in Desktop/src-tauri/tests/integration/test_export_import.rs
- [ ] T096 [P] [US7] Test data integrity across export/import

### Implementation for User Story 7

- [ ] T097 [US7] Implement export_wdr_report command in Desktop/src-tauri/src/commands/reports.rs
- [ ] T098 [US7] Implement export_comparison command
- [ ] T099 [US7] Implement data import validation
- [ ] T100 [US7] Add export file format support (JSON, CSV, PDF)
- [ ] T101 [US7] Implement data integrity checks
- [ ] T102 [US7] Register export commands in lib.rs
- [ ] T103 [US7] Update frontend export/import UI
- [ ] T104 [US7] Add file system operations via Tauri

**Checkpoint**: User Story 7 complete - data export/import functional

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T105 [P] Update frontend services to replace all mock API calls with Tauri IPC
- [ ] T106 [P] Run cargo test and achieve 100% coverage per Constitution IX
- [ ] T107 [P] Run cargo clippy and fix all linting issues
- [ ] T108 [P] Run cargo fmt to format all code
- [ ] T109 [P] Update documentation in docs/ directory
- [ ] T110 [P] Performance optimization: Add database indexes and query optimization
- [ ] T111 [P] Add progress reporting for long-running operations per SC-009
- [ ] T112 [P] Implement menu bar with "导入/导出PNG" per Constitution VIII
- [ ] T113 [P] Verify Chrome 86.x compatibility for all frontend code
- [ ] T114 [P] Create integration tests for end-to-end workflows
- [ ] T115 [P] Add error boundary and recovery mechanisms
- [ ] T116 [P] Validate all success criteria (SC-001 through SC-010)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-9)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 10)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational (Phase 2) - May integrate with US1 but independently testable
- **User Story 3 (P1)**: Can start after Foundational (Phase 2) - May integrate with US1/US2 but independently testable
- **User Story 4 (P2)**: Can start after Foundational (Phase 2) - Depends on US2 (needs reports to compare)
- **User Story 5 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 6 (P3)**: Can start after Foundational (Phase 2) - Depends on US2 (needs reports to audit)
- **User Story 7 (P3)**: Can start after Foundational (Phase 2) - No dependencies on other stories

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models before services
- Services before commands/handlers
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes:
  - User Stories 1, 2, 3, 5 can start in parallel (P1/P2, independently testable)
  - User Stories 4 depends on US2, User Story 6 depends on US2
- All tests for a user story marked [P] can run in parallel
- Models within a story marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for get_instance_summaries in Desktop/src-tauri/tests/dashboard_test.rs"
Task: "Unit test for get_dashboard_metrics in Desktop/src-tauri/tests/dashboard_test.rs"
Task: "Integration test for dashboard data loading in Desktop/src-tauri/tests/integration/test_dashboard.rs"

# Launch all models for User Story 1 together:
Task: "Implement dashboard models (InstanceSummary, DashboardMetrics) in Desktop/src-tauri/src/models/dashboard.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Continue with P2 stories (4, 5)
6. Complete with P3 stories (6, 7)
7. Polish phase for quality

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (P1)
   - Developer B: User Story 2 (P1)
   - Developer C: User Story 3 (P1)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Constitution compliance must be verified at each phase
- All 15 functional requirements (FR-001 through FR-015) must be implemented across user stories
- All 10 success criteria (SC-001 through SC-010) must be validated
- 100% test coverage is mandatory per Constitution Principle IX
