# Feature Specification: Implement Desktop Tauri Backend

**Feature Branch**: `001-implement-desktop`
**Created**: 2025-12-22
**Status**: Draft
**Input**: User description: "请实现Desktop。当前Desktop的前台界面已经实现了，是基于react的。Desktop应基于tauri实现。当前项目原型在Desktop下，已经可以正常启动、运行。需要按照docs下文档的要求，实现tauri的后台服务，替换掉前台的mock调用，使前后台能集成运行"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Launch Desktop Application (Priority: P1)

Users start the WDRProbe desktop application and see the dashboard with real data loaded from the backend service.

**Why this priority**: This is the entry point for all users - the application must successfully start and display data before any other functionality can be used.

**Independent Test**: Can be fully tested by launching the desktop application and verifying the dashboard displays instance summaries, metrics, and recent reports without errors.

**Acceptance Scenarios**:

1. **Given** the desktop application is installed, **When** a user launches it, **Then** the application window opens and displays the dashboard with loading indicators followed by actual data from the Tauri backend.
2. **Given** the application is running, **When** a user views the dashboard, **Then** they see instance summaries, dashboard metrics, hot issues, and recent reports populated from the backend database.
3. **Given** the application starts for the first time, **When** the backend initializes, **Then** it creates the local SQLite database with the required schema and shows an empty state or welcome message if no data exists.

---

### User Story 2 - Import and Manage WDR Reports (Priority: P1)

Users import WDR report files (HTML or raw format) through the desktop application, and the backend parses and stores them for analysis.

**Why this priority**: WDR report import is core functionality - users must be able to import their database performance reports to analyze them.

**Independent Test**: Can be fully tested by importing a WDR file via drag-and-drop or file picker, verifying it appears in the reports list, and can be viewed in detail.

**Acceptance Scenarios**:

1. **Given** the desktop application is running, **When** a user drags and drops a WDR HTML file into the application, **Then** the backend parses the file, extracts SQL statistics and performance metrics, stores them in SQLite, and displays a success notification.
2. **Given** reports have been imported, **When** a user views the reports list, **Then** they see all imported reports with metadata (ID, instance name, generation time, status) loaded from the backend.
3. **Given** a user selects a report from the list, **When** they view the report details, **Then** the backend retrieves and displays efficiency metrics, load profile, top SQL statistics, and object statistics from the parsed data.
4. **Given** a user deletes a report, **When** they confirm the deletion, **Then** the backend removes the report and all associated data from the database and updates the UI.

---

### User Story 3 - Analyze SQL Execution Plans (Priority: P1)

Users view and interact with execution plan visualizations, with hot SQL queries automatically opening execution plan views.

**Why this priority**: Execution plan analysis is the primary value proposition - users need to understand why SQL queries are slow and how to optimize them.

**Independent Test**: Can be fully tested by clicking on a hot SQL query from a WDR report and verifying the execution plan tree renders with cost information and optimization suggestions.

**Acceptance Scenarios**:

1. **Given** a user is viewing WDR report details with Top SQL list, **When** they click on a SQL entry, **Then** the execution plan visualizer opens and displays the plan tree with operator types, cost values, and row estimates.
2. **Given** an execution plan is displayed, **When** a user hovers over or clicks nodes in the tree, **Then** detailed information appears (output columns, filter conditions, buffer usage) retrieved from the backend's plan analysis.
3. **Given** the backend detects high-cost operations (e.g., sequential scans with cost > threshold), **When** the plan is rendered, **Then** those nodes are highlighted with warnings and optimization suggestions are shown.
4. **Given** a user manually pastes SQL text, **When** they request the execution plan, **Then** the backend parses the SQL, generates or retrieves the plan structure, and displays it in the visualizer.

---

### User Story 4 - Compare WDR Reports (Priority: P2)

Users select two WDR reports and compare their performance metrics to identify degradation or improvements over time.

**Why this priority**: Performance comparison is a critical diagnostic tool for identifying when and why database performance changed.

**Independent Test**: Can be fully tested by selecting two reports from the list, running a comparison, and viewing the results showing metric changes and key findings.

**Acceptance Scenarios**:

1. **Given** at least two WDR reports exist in the system, **When** a user selects two reports and initiates a comparison, **Then** the backend calculates differences for all metrics (CPU time, IO, execution counts) and displays a summary with overall performance score change.
2. **Given** a comparison is in progress, **When** the backend processes the data, **Then** it identifies key findings (e.g., "CPU time increased by 45%", "Wait events doubled") and presents them in a structured format.
3. **Given** comparison results are displayed, **When** a user clicks on a specific category (SQL, wait events, objects, or system metrics), **Then** the backend provides detailed comparison data with before/after values and percentage changes.
4. **Given** comparison data is available, **When** a user views SQL-level comparison, **Then** they see the same SQL's performance metrics side-by-side with change indicators (arrows, color coding).

---

### User Story 5 - Configure Performance Thresholds (Priority: P2)

Users adjust threshold values for various performance metrics to customize alerts and analysis sensitivity.

**Why this priority**: Thresholds allow users to customize the tool to their specific environment and performance expectations.

**Independent Test**: Can be fully tested by updating a threshold value and verifying it affects highlighting in reports and comparisons.

**Acceptance Scenarios**:

1. **Given** the threshold configuration page is open, **When** a user modifies a threshold value and saves, **Then** the backend validates the value, updates it in the database, and applies it to all future analysis.
2. **Given** thresholds are updated, **When** a user views a WDR report or comparison, **Then** the highlighting and alerts reflect the new threshold values.
3. **Given** a user applies a threshold template (e.g., "High Concurrency"), **When** they select it, **Then** the backend applies the preset threshold values for all categories in one operation.
4. **Given** threshold changes are made, **When** they are saved, **Then** the backend records audit information (user, timestamp, reason) for compliance tracking.

---

### User Story 6 - View SQL Audit Results (Priority: P3)

Users review automatically detected SQL performance issues and optimization recommendations.

**Why this priority**: Automated SQL auditing provides proactive identification of performance problems.

**Independent Test**: Can be fully tested by viewing the SQL audit page and seeing detected issues with their severity levels and recommendations.

**Acceptance Scenarios**:

1. **Given** WDR reports are loaded in the system, **When** a user visits the SQL audit page, **Then** the backend scans all reports for issues (full table scans, missing indexes, inefficient joins) and displays a prioritized list.
2. **Given** audit issues are displayed, **When** a user selects an issue, **Then** they see detailed information including the problematic SQL, the specific problem, and concrete optimization recommendations (e.g., "Create index on column X").
3. **Given** an issue is reviewed, **When** a user marks it as "whitelisted" or "fixed", **Then** the backend updates the issue status and no longer flags it in future audits.

---

### User Story 7 - Export and Import Data (Priority: P3)

Users export reports, comparisons, and configurations, and import them on another machine or after reinstalling the application.

**Why this priority**: Data portability allows users to backup their analysis and share it with team members.

**Independent Test**: Can be fully tested by exporting a report and then importing it into a fresh application instance, verifying all data is preserved.

**Acceptance Scenarios**:

1. **Given** reports exist in the system, **When** a user selects export and chooses a location, **Then** the backend creates an export file containing the selected reports and their data.
2. **Given** an export file exists, **When** a user imports it into the application, **Then** the backend validates the file format, imports all data into SQLite, and updates the UI to reflect the new data.
3. **Given** a user exports a comparison, **When** they choose export options, **Then** the backend includes all related report data and comparison results in the export file.

---

### Edge Cases

- What happens when importing a malformed or corrupted WDR file?
- How does the system handle very large WDR files (>100MB)?
- What occurs when the backend database is corrupted or inaccessible?
- How are threshold updates handled when multiple users modify them simultaneously?
- What happens when viewing an execution plan for a SQL query that has been deleted from the source report?
- How does the system respond when disk space is insufficient for imports or exports?
- What occurs when comparing reports from different database instances with incompatible metrics?
- How are long-running operations (parsing large files, complex comparisons) handled if the user closes the application?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide Tauri backend commands for all IPC interfaces defined in desktop-IPC.md (get_instance_summaries, get_wdr_reports, get_execution_plan, etc.)
- **FR-002**: System MUST parse WDR report files (HTML and raw formats) and extract SQL statistics, performance metrics, and object statistics for storage in SQLite
- **FR-003**: System MUST implement SQLite database with schema for storing WDR reports, comparison data, threshold configurations, and audit logs
- **FR-004**: System MUST provide file import/export functionality for WDR reports, comparison results, and configuration data via Tauri IPC
- **FR-005**: System MUST support execution plan parsing and visualization data generation for both stored WDR data and user-provided SQL text
- **FR-006**: System MUST calculate performance metric differences between two WDR reports and generate intelligent analysis summaries
- **FR-007**: System MUST manage threshold configurations with CRUD operations and apply them to analysis and highlighting
- **FR-008**: System MUST perform automated SQL auditing to identify performance issues (full table scans, missing indexes, inefficient operations)
- **FR-009**: System MUST maintain audit logs for all user actions (threshold updates, report deletions, configuration changes)
- **FR-010**: System MUST integrate with GaussDB for EXPLAIN plan analysis using FORMAT JSON and EXPLAIN ANALYZE compatibility
- **FR-011**: System MUST use enable_hypo_index parameter simulation for virtual index evaluation during plan analysis
- **FR-012**: System MUST provide IPC commands for dashboard metrics aggregation (CPU, memory, TPS, QPS) from stored WDR data
- **FR-013**: System MUST support hot SQL query detection and automatic linkage to execution plan views
- **FR-014**: System MUST implement responsive backend operations with progress reporting for long-running tasks (file parsing, comparisons)
- **FR-015**: System MUST validate all inputs (file formats, threshold values, SQL text) and provide meaningful error messages

### Key Entities

- **WDR Report**: Contains metadata (ID, instance name, generation time, snapshot range) and performance data (efficiency metrics, load profile, top SQL statistics, object statistics)
- **Execution Plan**: Tree structure representing SQL execution with operator nodes, cost values, row estimates, and optimization hints
- **Comparison Result**: Contains before/after metrics, percentage changes, key findings, and overall performance score difference
- **Threshold Configuration**: Categorized performance thresholds (SQL, Wait, System, AI) with values, update timestamps, and change history
- **SQL Audit Issue**: Detected performance problem with severity level, problematic SQL text, problem description, and optimization recommendation
- **Audit Log**: Records user actions with timestamp, operation type, affected data, and change details

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can launch the desktop application and view the dashboard with real backend data within 3 seconds of application startup
- **SC-002**: WDR report import completes for files up to 50MB within 30 seconds with 100% accuracy in data extraction
- **SC-003**: Execution plan visualization renders for any stored or user-provided SQL within 2 seconds with accurate cost and operator information
- **SC-004**: Report comparison completes for typical WDR reports (1000+ SQL entries) within 10 seconds and presents clear performance change analysis
- **SC-005**: Threshold configuration changes are applied to all future analysis within 1 second of saving
- **SC-006**: SQL audit identifies at least 90% of common performance issues (full table scans, missing indexes, inefficient joins)
- **SC-007**: Backend maintains 99.9% data integrity for imported reports with no data loss or corruption
- **SC-008**: Application responds to all user actions within 1 second for standard operations (viewing data, navigating pages)
- **SC-009**: Long-running operations (import, comparison) provide progress updates every 2 seconds and can be cancelled
- **SC-010**: Exported data can be successfully imported on another machine with 100% data fidelity

## Assumptions

- Frontend React application is fully implemented with mock API calls that need to be replaced with Tauri IPC invocations
- SQLite will be used for local data storage to maintain simplicity and eliminate external database dependencies
- WDR report files will be in standard HTML format or the proprietary raw format documented in gaussdb.md
- Users have GaussDB database access for executing EXPLAIN commands when analyzing user-provided SQL text
- Desktop application runs on Windows, macOS, or Linux with Tauri runtime support
- File import/export operations will use native file system APIs through Tauri
- Performance thresholds default to industry-standard values for database performance analysis
- The application operates in offline mode with no cloud synchronization required

## Dependencies

- Tauri framework for desktop application runtime and IPC communication
- SQLite embedded database for local data storage
- React frontend application (already implemented) with services layer for API calls
- GaussDB database connectivity for EXPLAIN plan execution
- File system access for WDR report import/export operations
- Local configuration storage for user preferences and threshold settings
