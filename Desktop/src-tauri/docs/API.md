# WDRProbe Desktop - API Reference

Complete reference for all Tauri IPC commands exposed to the frontend.

## Table of Contents

- [Dashboard Commands](#dashboard-commands)
- [Report Management Commands](#report-management-commands)
- [Execution Plan Commands](#execution-plan-commands)
- [Comparison Commands](#comparison-commands)
- [Threshold Commands](#threshold-commands)
- [Audit Commands](#audit-commands)
- [Export/Import Commands](#exportimport-commands)

---

## Dashboard Commands

### `getInstanceSummaries`

Get list of all database instances with summary metrics.

**Returns:** `Promise<InstanceSummary[]>`

```typescript
interface InstanceSummary {
  instanceName: string;
  reportCount: number;
  latestReportTime: string | null;
  totalSqlCount: number;
}
```

**Example:**
```typescript
const instances = await invoke('get_instance_summaries');
// [{ instanceName: "primary", reportCount: 5, ... }]
```

---

### `getDashboardMetrics`

Get dashboard statistics for a specific instance or all instances.

**Parameters:**
- `instanceName?: string` - Filter by instance name (optional)

**Returns:** `Promise<DashboardMetrics>`

```typescript
interface DashboardMetrics {
  totalReports: number;
  totalSqlCount: number;
  instanceCount: number;
  recentImports: WdrReportSummary[];
  avgSqlCount: number;
}
```

---

## Report Management Commands

### `importWdrReport`

Import a WDR HTML file into the database.

**Parameters:**
- `filePath: string` - Path to WDR HTML file
- `instanceName: string` - Name of the database instance

**Returns:** `Promise<ImportResult>`

```typescript
interface ImportResult {
  success: boolean;
  reportId: number | null;
  sqlCount: number;
  message: string;
}
```

---

### `getWdrReports`

List WDR reports with pagination.

**Parameters:**
- `limit?: number` - Max records to return (default: 50)
- `offset?: number` - Records to skip (default: 0)
- `sortBy?: string` - Sort field (default: "generationTime")

**Returns:** `Promise<WdrReportListResponse>`

```typescript
interface WdrReportListResponse {
  reports: WdrReportSummary[];
  total: number;
}

interface WdrReportSummary {
  id: number;
  instanceName: string;
  snapshotTime: string;
  generationTime: string;
  sqlCount: number;
  totalElapsedTime: number;
  status: string;
}
```

---

### `getWdrReportDetail`

Get detailed information about a specific report.

**Parameters:**
- `reportId: number` - Report ID

**Returns:** `Promise<WdrReportDetail>`

```typescript
interface WdrReportDetail {
  report: WdrReportSummary;
  instanceInfo: InstanceInfo;
  sqlList: TopSql[];
  loadProfile: LoadProfile | null;
  efficiencyMetrics: EfficiencyMetrics | null;
}
```

---

### `deleteWdrReport`

Delete a WDR report and associated data.

**Parameters:**
- `reportId: number` - Report ID to delete

**Returns:** `Promise<DeleteResult>`

```typescript
interface DeleteResult {
  success: boolean;
  deletedReportId: number;
  message?: string;
}
```

---

## Execution Plan Commands

### `getWdrHotSqls`

Get top SQL statements from a WDR report.

**Parameters:**
- `reportId: number` - Report ID
- `limit?: number` - Max records (default: 100)

**Returns:** `Promise<TopSql[]>`

```typescript
interface TopSql {
  id: number;
  reportId: number;
  rank: number;
  sqlText: string;
  executions: number;
  totalElapsedTime: number;
  cpuTime: number;
  ioTime: number;
  bufferGets: number;
  diskReads: number;
  rowsProcessed: number;
}
```

---

### `getExecutionPlan`

Get the execution plan for a SQL statement.

**Parameters:**
- `sqlId: number` - SQL ID

**Returns:** `Promise<SqlExecutionPlan>`

```typescript
interface SqlExecutionPlan {
  id: number | null;
  sqlId: number;
  planJson: string;
  planTree: ExecutionPlanNode | null;
  optimizationSuggestions: string[];
}
```

---

### `analyzeExecutionPlan`

Analyze an execution plan and provide optimization suggestions.

**Parameters:**
- `sqlId: number` - SQL ID to analyze

**Returns:** `Promise<PlanAnalysisResult>`

```typescript
interface PlanAnalysisResult {
  sqlId: number;
  sqlText: string;
  planAnalysis: PlanAnalysis;
  suggestions: OptimizationSuggestion[];
}
```

---

## Comparison Commands

### `createComparison`

Create a new comparison between two WDR reports.

**Parameters:**
- `sourceReportId: number` - Source/baseline report ID
- `targetReportId: number` - Target/new report ID
- `comparisonType?: string` - "TimeBased", "InstanceBased", or "AdHoc"
- `customName?: string` - Custom comparison name

**Returns:** `Promise<CreateComparisonResult>`

```typescript
interface CreateComparisonResult {
  success: boolean;
  comparisonId: number;
  message: string;
  processingTimeMs: number;
}
```

---

### `getComparisons`

List comparisons with pagination.

**Parameters:**
- `limit?: number` - Max records
- `offset?: number` - Records to skip
- `sortBy?: string` - Sort field

**Returns:** `Promise<ComparisonListResponse>`

```typescript
interface ComparisonListResponse {
  comparisons: WdrComparisonListItem[];
  total: number;
}
```

---

### `getComparisonDetail`

Get detailed comparison data.

**Parameters:**
- `comparisonId: number` - Comparison ID

**Returns:** `Promise<WdrComparisonDetail>`

---

### `getComparisonMetrics`

Get comparison metrics for a specific category.

**Parameters:**
- `comparisonId: number` - Comparison ID
- `category: string` - "sql", "wait", "obj", "sys", "instance"
- `sortBy?: string` - Sort field
- `limit?: number` - Max records
- `offset?: number` - Records to skip

**Returns:** `Promise<ComparisonMetricsResponse>`

---

### `deleteComparison`

Delete a comparison.

**Parameters:**
- `comparisonId: number` - Comparison ID

**Returns:** `Promise<DeleteComparisonResult>`

---

## Threshold Commands

### `getThresholdConfigs`

Get all threshold configurations.

**Returns:** `Promise<ThresholdConfig[]>`

```typescript
interface ThresholdConfig {
  configKey: string;
  displayName: string;
  currentValue: number;
  minValue: number;
  maxValue: number;
  dataType: string;
  category: string;
  updatedAt: string;
}
```

---

### `getThresholdTemplate`

Get a predefined threshold template.

**Parameters:**
- `templateName: string` - Template name ("HighConcurrency", "LowResource", etc.)

**Returns:** `Promise<ThresholdConfig[]>`

---

### `updateThreshold`

Update one or more threshold configurations.

**Parameters:**
- `updates: ThresholdUpdateRequest[]` - Array of updates

**Returns:** `Promise<ThresholdUpdateResult>`

```typescript
interface ThresholdUpdateRequest {
  configKey: string;
  value: number;
  changedBy: string;
  changeReason: string;
}
```

---

### `resetToTemplate`

Reset thresholds to a template.

**Parameters:**
- `templateName: string` - Template name

**Returns:** `Promise<ThresholdUpdateResult>`

---

## Audit Commands

### `runSqlAudit`

Run SQL audit on one or more reports.

**Parameters:**
- `reportIds?: number[]` - Specific reports to audit (or all)
- `includeResolved?: boolean` - Include resolved issues
- `auditTypes?: string[]` - Specific audit types

**Returns:** `Promise<AuditRunResult>`

```typescript
interface AuditRunResult {
  success: boolean;
  reportsAudited: number;
  newIssuesFound: number;
  existingIssuesUpdated: number;
  issues: SqlAuditIssue[];
  durationMs: number;
  message?: string;
}
```

---

### `getSqlAuditIssues`

Get SQL audit issues with filtering.

**Parameters:**
- `reportId?: number` - Filter by report
- `status?: string` - Filter by status
- `severity?: string` - Filter by severity
- `issueType?: string` - Filter by type
- `limit?: number` - Max records
- `offset?: number` - Records to skip
- `sortBy?: string` - Sort field

**Returns:** `Promise<SqlAuditIssueList>`

---

### `updateAuditIssueStatus`

Update a single audit issue status.

**Parameters:**
- `issueId: number` - Issue ID
- `status: string` - New status ("Reviewed", "Fixed", "Whitelisted", "Ignored")
- `resolvedBy: string` - User making the change
- `resolutionNote?: string` - Optional note

**Returns:** `Promise<UpdateAuditIssueResult>`

---

### `bulkUpdateAuditIssues`

Bulk update multiple audit issues.

**Parameters:**
- `issueIds: number[]` - Issue IDs to update
- `status: string` - New status
- `resolvedBy: string` - User making the change
- `resolutionNote?: string` - Optional note

**Returns:** `Promise<BulkUpdateResult>`

---

### `getAuditSummary`

Get summary statistics for audit issues.

**Returns:** `Promise<AuditSummary>`

```typescript
interface AuditSummary {
  totalIssues: number;
  bySeverity: Record<string, number>;
  byStatus: Record<string, number>;
  byType: Record<string, number>;
}
```

---

## Export/Import Commands

### `exportWdrReport`

Export a WDR report to file.

**Parameters:**
- `reportId: number` - Report to export
- `format: string` - "Json", "Csv", or "Pdf"
- `includeSqlDetails: boolean` - Include SQL data
- `includeComparisonData: boolean` - Include comparison data
- `exportPath?: string` - Custom file path

**Returns:** `Promise<ExportResult>`

```typescript
interface ExportResult {
  success: boolean;
  exportPath: string;
  recordCount: number;
  fileSize: number;
  format: string;
  message?: string;
}
```

---

### `exportComparison`

Export comparison data to file.

**Parameters:**
- `comparisonId: number` - Comparison to export
- `format: string` - "Json", "Csv", or "Pdf"
- `exportPath?: string` - Custom file path

**Returns:** `Promise<ExportResult>`

---

### `importData`

Import data from an export file.

**Parameters:**
- `importPath: string` - Path to import file
- `validateOnly: boolean` - Validate without importing
- `overwriteExisting: boolean` - Overwrite existing records
- `importTypes: string[]` - Types to import ("Reports", "Comparisons", "Thresholds", "AuditIssues")

**Returns:** `Promise<ImportResult>`

```typescript
interface ImportResult {
  success: boolean;
  recordsImported: number;
  recordsSkipped: number;
  recordsFailed: number;
  warnings: string[];
  errors: string[];
  validationErrors: string[];
  message?: string;
}
```

---

### `validateDataIntegrity`

Validate data integrity using checksums or record counts.

**Parameters:**
- `checkType: string` - "Checksum", "RecordCount", or "SchemaValidation"
- `entityType: string` - Entity type to validate
- `entityId?: number` - Specific entity ID
- `expectedHash?: string` - Expected hash value

**Returns:** `Promise<DataIntegrityCheck>`

```typescript
interface DataIntegrityCheck {
  checkType: string;
  entityType: string;
  entityId: number | null;
  expectedHash: string | null;
  actualHash: string | null;
  passed: boolean;
  message?: string;
}
```

---

## Error Handling

All commands return errors as strings. Check for errors in frontend:

```typescript
try {
  const result = await invoke('get_wdr_reports', { limit: 50 });
  // Handle success
} catch (error) {
  // error is a string describing the failure
  console.error('Command failed:', error);
}
```

---

## Type Definitions

TypeScript types are auto-generated from Rust structs. Refer to the Rust source files in `src/models/` for complete type definitions.
