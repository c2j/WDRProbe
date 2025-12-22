# WDRProbe Constitution
<!-- WDRProbe: Database Query Performance Analysis Tool -->

<!-- Sync Impact Report
Version: 0.0.0 → 1.0.0 (MAJOR)
Added: 9 Core Principles covering React architecture, API design, GaussDB compatibility, WDR integration, performance optimization, and desktop adaptation
Templates Updated: ✅ All checked - plan-template.md, spec-template.md, tasks-template.md (no commands directory)
Follow-up TODOs: TODO(RATIFICATION_DATE): Set original adoption date when known
-->

## Core Principles

### I. React Component-First Architecture (NON-NEGOTIABLE)
All UI must be constructed using React functional or class components exclusively. Mixing with Vue or other frameworks is prohibited. Each component must be independent, reusable, and strictly adhere to hooks rules to prevent side effect leaks. Component boundaries must be clear and well-defined with single responsibilities.

### II. Frontend Style Compatibility
Code must maintain compatibility with low-version Chrome browsers (86.x). Avoid using modern CSS features or JavaScript APIs not available in Chrome 86. Test all UI components against the minimum supported browser version before deployment. Progressive enhancement must be applied for newer features.

### III. Unified API Encapsulation
All frontend-backend interactions must use specific ApiService methods. Direct use of axios or generic appStore.apiService methods is prohibited. All API responses must unpack the `data` field consistently, and error handling must use ElMessage component. No direct HTTP client usage outside the ApiService abstraction layer.

### IV. Threshold Configuration DTO
Threshold management must use backend DTOs containing: category, dataType, configKey, and value fields. Frontend must remove all local threshold mappings. Update interfaces must统一采用 `{ value: number, changedBy, changeReason }` format. All threshold changes require audit trail with user attribution and reason.

### V. GaussDB Compatibility Testing
All SQL parsing and EXPLAIN processing must be compatible with GaussDB syntax. Reference gaussdb.md for EXPLAIN ANALYZE and FORMAT JSON specifications. During development, enable_hypo_index must be used to simulate virtual index evaluation. Database compatibility tests must cover all GaussDB-specific features and edge cases.

### VI. WDR Seamless Integration
Hot SQL queries must automatically trigger the execution plan view when clicked. WDR data must联动 (link) with execution plans, such as triggering table scan alerts when Rows Scanned > 1e6. Threshold adjustment buttons must directly call the updateThresholdConfig API. User interactions must be smooth and require zero manual data transfer between WDR and execution plan views.

### VII. Performance Optimization Rules
Code must apply gaussdb.md parameter guidance (such as shared_buffers tuning recommendations). Visualization rendering must limit virtual DOM usage. Complex execution plan trees must NOT use synchronous traversal - child nodes must load asynchronously. Performance budgets must be defined and monitored for all rendering operations.

### VIII. Tauri Desktop Adaptation
Desktop端 must use IPC communication for file import operations (WDR/EXPLAIN files). Menu bar must强制包含 "导入/导出PNG" (Import/Export PNG) options. Layout must use Flexbox with left SQL editor at 30% width and right tree view at 50% width. Desktop-specific optimizations must be implemented for file handling and native menu integration.

### IX. Documentation and Test Coverage
Every functional point must同步更新 design.md-style documentation with wireframes and algorithm examples. Unit tests must cover 100% of API endpoints and components. Commits without tests are prohibited. Test coverage reports must be generated and reviewed for every PR. Documentation must include visual examples and use case scenarios.

## Technical Constraints

**Technology Stack Requirements:**
- Frontend: React (functional/class components only)
- Desktop Framework: Tauri with IPC communication
- Database: GaussDB with specific EXPLAIN format support
- Error Handling: ElMessage for all user-facing errors
- Styling: Chrome 86.x compatible CSS/JS only

**Performance Requirements:**
- Async loading for complex visualization trees
- Virtual DOM usage limitations
- Shared buffers and memory optimization per gaussdb.md
- File import/export via IPC for desktop builds

**Compatibility Requirements:**
- Chrome 86.x minimum browser version
- GaussDB-specific SQL and EXPLAIN syntax
- WDR report format integration
- Threshold DTO schema compliance

## Development Workflow

**Code Review Requirements:**
- Verify React component isolation and reusability
- Confirm Chrome 86.x compatibility for all UI changes
- Validate ApiService encapsulation usage (no direct HTTP clients)
- Check test coverage meets 100% requirement
- Ensure documentation updates accompany all features

**Testing Gates:**
- All API endpoints must have unit tests
- All React components must have unit tests
- GaussDB compatibility tests required for SQL changes
- WDR integration tests for execution plan features
- Desktop file handling tests for Tauri builds

**Quality Gates:**
- No direct axios or generic API calls allowed
- No synchronous traversal of complex plan trees
- No threshold changes without DTO format
- No commits without corresponding test coverage
- No features without design.md documentation updates

## Governance

**Constitutional Authority:**
This constitution supersedes all other development practices and guidelines. When conflicts arise between this constitution and other documentation, this document takes precedence.

**Amendment Procedure:**
Constitution amendments require:
1. Documentation update with version bump justification
2. Impact assessment on existing codebase
3. Migration plan for breaking changes
4. PR review with explicit constitutional compliance check
5. Version increment per semantic versioning rules (MAJOR for principle removals/redefinitions, MINOR for additions, PATCH for clarifications)

**Compliance Review:**
All PRs must explicitly verify constitutional compliance. Template checklists in plan-template.md, spec-template.md, and tasks-template.md must be completed before merge. Violations require complexity justification and approval from maintainers.

**Versioning Policy:**
- MAJOR: Backward-incompatible governance changes, principle removals, or redefinitions
- MINOR: New principles added or materially expanded guidance
- PATCH: Clarifications, wording fixes, non-semantic refinements

**Version**: 1.0.0 | **Ratified**: TODO(RATIFICATION_DATE) | **Last Amended**: 2025-12-22
