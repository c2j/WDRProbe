import React, {
  createContext,
  useState,
  useContext,
  ReactNode,
  useEffect,
} from "react";

type Language = "en" | "zh";

const translations = {
  en: {
    // Menu
    "menu.dashboard": "Dashboard",
    "menu.reports": "WDR Reports",
    "menu.comparison": "Comparison",
    "menu.visualizer": "Plan Visualizer",
    "menu.thresholds": "Thresholds",
    "menu.sqlaudit": "SQL Audit",
    "menu.auditlog": "Audit Logs",

    // Header
    "header.role": "DBA Admin",

    // Dashboard
    "dash.cpu": "CPU Usage",
    "dash.mem": "Memory Usage",
    "dash.tps": "TPS",
    "dash.qps": "QPS",
    "dash.health": "Instance Health",
    "dash.trend": "Top Issue Trend (Today)",
    "dash.hotIssues": "Top 10 Hot Issues",
    "dash.recentReports": "Recent WDR Reports",
    "dash.view": "View",
    "dash.details": "Details",
    "dash.healthy": "Healthy",
    "dash.warning": "Warning",
    "dash.critical": "Critical",
    "dash.instanceOverview": "Instance Overview",
    "dash.selectInstance": "Select Instance",
    "dash.allInstances": "All Instances",
    "dash.lastReport": "Last Report",
    "dash.activeIssues": "Active Issues",
    "dash.score": "Health Score",

    // Reports
    "rep.search": "Search Instance...",
    "rep.upload": "Upload Report",
    "rep.importReport": "Import Report",
    "rep.id": "ID",
    "rep.instance": "Instance",
    "rep.generated": "Generated At",
    "rep.period": "Period",
    "rep.status": "Status",
    "rep.actions": "Actions",
    "rep.loading": "Loading...",
    "rep.uploadTitle": "Import WDR Report",
    "rep.instanceName": "Instance Name",
    "rep.description": "Description",
    "rep.file": "Report File",
    "rep.filePath": "File Path",
    "rep.fileRequired": "File path is required",
    "rep.instanceRequired": "Instance name is required",
    "rep.fileHint": "HTML or WDR files up to 50MB",
    "rep.cancel": "Cancel",
    "rep.submit": "Import Report",
    "rep.import": "Import",
    "rep.uploading": "Uploading...",
    "rep.uploadFailed": "Upload failed",
    "rep.showing": "Showing {start}-{end} of {total}",
    "rep.prev": "Prev",
    "rep.next": "Next",
    "rep.deleteTitle": "Delete Report",
    "rep.deleteConfirm":
      "Are you sure you want to delete report #{id}? This action cannot be undone.",
    "rep.delete": "Delete",
    "rep.viewTitle": "Report Details",
    "rep.tab.summary": "Overview",
    "rep.tab.load": "Load Profile",
    "rep.tab.sqlstats": "SQL Statistics",
    "rep.tab.objstats": "Object Statistics",
    "rep.summary.efficiency": "Instance Efficiency",
    "rep.summary.workload": "Workload Profile (Per Second)",
    "rep.summary.db": "Database Statistics",
    "rep.metric": "Metric",
    "rep.perSec": "Per Second",
    "rep.perTxn": "Per Transaction",
    "rep.sql.orderBy": "Order By",
    "rep.sql.elapsed": "Elapsed Time",
    "rep.sql.cpu": "CPU Time",
    "rep.sql.calls": "Executions",
    "rep.sql.reads": "Physical Reads",
    "rep.sql.text": "Full SQL Text",
    "rep.sql.visualize": "Visualize Plan",
    "rep.obj.table": "Tables",
    "rep.obj.index": "Indexes",
    "rep.obj.deadTup": "Dead Tuples",
    "rep.obj.liveTup": "Live Tuples",
    "rep.obj.seqScan": "Seq Scan",
    "rep.obj.idxScan": "Idx Scan",

    // Comparison
    "comp.selected": "Selected Reports",
    "comp.new": "New Comparison",
    "comp.name": "Comparison Name",
    "comp.desc": "Description",
    "comp.save": "Save",
    "comp.tab.sql": "SQL Statistics",
    "comp.tab.wait": "Wait Events",
    "comp.tab.obj": "Object Stats",
    "comp.tab.sys": "System Metrics",
    "comp.fingerprint": "SQL Fingerprint",
    "comp.r1": "Report 1",
    "comp.r2": "Report 2",
    "comp.change": "Change",
    "comp.plan": "Plan",
    "comp.action": "Action",
    "comp.col.event": "Event Name",
    "comp.col.class": "Class",
    "comp.col.time": "Time (ms)",
    "comp.col.object": "Object Name",
    "comp.col.schema": "Schema",
    "comp.col.scans": "Scans",
    "comp.col.metric": "Metric Name",
    "comp.col.val": "Value",
    "comp.col.diff": "Diff",
    "comp.col.cpu": "CPU Time",
    "comp.col.io": "IO Time",
    "comp.col.seq": "Seq Scan",
    "comp.col.idx": "Idx Scan",
    "comp.col.tup": "Tuples (I/U/D)",
    "comp.overview.title": "Metric Comparison Overview",
    "comp.chart.dbTime": "DB Time (s)",
    "comp.chart.cpu": "CPU Usage (%)",
    "comp.chart.io": "IOPS",
    "comp.col.phyRd": "Phy Reads",
    "comp.col.logRd": "Log Reads",
    "comp.col.heapRd": "Heap Read",
    "comp.col.heapHit": "Heap Hit",
    "comp.col.idxRd": "Idx Read",
    "comp.col.idxHit": "Idx Hit",

    // Comparison Summary
    "comp.summary.title": "Analysis Summary",
    "comp.summary.conclusion": "Conclusion",
    "comp.summary.findings": "Key Findings",
    "comp.summary.score": "Performance Score",
    "comp.status.improved": "Improved",
    "comp.status.degraded": "Degraded",
    "comp.status.stable": "Stable",

    // Thresholds
    "thr.categories": "Categories",
    "thr.config": "Configuration",
    "thr.batchSave": "Batch Save",
    "thr.templates": "Templates",
    "thr.sysTemplates": "System Templates",
    "thr.customTemplates": "Custom Templates",
    "thr.key": "Key",
    "thr.value": "Value",
    "thr.unit": "Unit",
    "thr.range": "Range",
    "thr.action": "Action",

    // SQL Audit
    "audit.all": "All",
    "audit.pending": "Pending",
    "audit.processing": "Processing",
    "audit.fixed": "Fixed",
    "audit.whitelisted": "Whitelisted",
    "audit.id": "ID",
    "audit.severity": "Severity",
    "audit.type": "Type",
    "audit.target": "Target",
    "audit.foundTime": "Found Time",
    "audit.status": "Status",
    "audit.actions": "Actions",
    "audit.optimize": "Optimize",
    "audit.modalTitle": "SQL Optimization",
    "audit.issueId": "Issue ID",
    "audit.targetSql": "Target SQL",
    "audit.diagnosis": "Diagnosis",
    "audit.recommendation": "Recommendation",
    "audit.apply": "Apply Fix",
    "audit.whitelist": "Whitelist",
    "audit.cancel": "Cancel",

    // Audit Log
    "log.allOps": "All Operations",
    "log.updateThr": "Update Threshold",
    "log.export": "Export",
    "log.time": "Time",
    "log.user": "User",
    "log.op": "Operation",
    "log.target": "Target",
    "log.result": "Result",

    // Visualizer
    "vis.title": "Execution Plan Visualizer",
    "vis.import": "Import",
    "vis.view": "View",
    "vis.costThreshold": "Cost Threshold",
    "vis.explain": "Explain",
    "vis.help": "Guide",
    "vis.sqlEditor": "SQL Editor",
    "vis.syntax": "GaussDB Syntax",
    "vis.pastePlaceholder": "Paste your SQL here...",
    "vis.planText": "Plan Text",
    "vis.rawExplain": "Raw Explain",
    "vis.noPlan": "No plan generated",
    "vis.visualTree": "Visual Tree",
    "vis.totalCost": "Total Cost",
    "vis.analyzing": "Analyzing Execution Plan...",
    "vis.selectSql": "Select a SQL to visualize plan",
    "vis.node.cost": "Cost",
    "vis.node.rows": "Rows",
    "vis.node.width": "Width",
    "vis.node.target": "Target Object",
    "vis.node.details": "Details",
    "vis.opt.suggestions": "Optimization Suggestions",
    "vis.opt.highCost": "High Cost Join Detected",
    "vis.opt.highCostDesc":
      "The Hash Join at the root has a cost of {cost}, which exceeds the threshold. Consider checking if the join columns are indexed.",
    "vis.opt.indexOpp": "Index Opportunity",
    "vis.opt.indexOppDesc":
      "Seq Scan on 'users' with filter. Consider creating an index.",
    "vis.opt.loadToSee": "Load a plan to see suggestions",
    "vis.hot.title": "WDR Hot SQLs",
    "vis.hot.viewFull": "View Full Report",
    "vis.hot.preview": "SQL Preview",
    "vis.hot.time": "Time",
    "vis.hot.cost": "Cost",
    "vis.hot.action": "Action",
    "vis.hot.load": "Load",

    // Knowledge Base
    "vis.kb.title": "Operator Knowledge",
    "vis.kb.search": "Search operators...",
    "vis.kb.pros": "When to use (Pros)",
    "vis.kb.cons": "Performance Risks (Cons)",

    // KB Entries
    "vis.kb.seqScan.title": "Seq Scan (Sequential Scan)",
    "vis.kb.seqScan.desc":
      "Reads every row in the table sequentially from beginning to end.",
    "vis.kb.seqScan.pros":
      "Efficient for small tables or when a query needs to fetch a large portion (>20%) of the table data.",
    "vis.kb.seqScan.cons":
      "Very slow on large tables if only a few rows are needed. High I/O consumption.",

    "vis.kb.idxScan.title": "Index Scan",
    "vis.kb.idxScan.desc":
      "Traverses a B-Tree index to find specific row locations, then fetches the actual data from the table heap.",
    "vis.kb.idxScan.pros":
      "Extremely fast for retrieving a small number of rows (high selectivity).",
    "vis.kb.idxScan.cons":
      "Becomes slower than Seq Scan if too many random I/O operations are required (low selectivity).",

    "vis.kb.idxOnlyScan.title": "Index Only Scan",
    "vis.kb.idxOnlyScan.desc":
      "Retrieves data directly from the index without visiting the table heap (Covering Index).",
    "vis.kb.idxOnlyScan.pros":
      "The fastest scanning method. Eliminates random table I/O completely.",
    "vis.kb.idxOnlyScan.cons":
      "Requires all requested columns to be present in the index. May still check heap for visibility (VM).",

    "vis.kb.bitmapScan.title": "Bitmap Scan (Heap/Index)",
    "vis.kb.bitmapScan.desc":
      "Builds a bitmap of matching pages from an index, sorts them, and then reads the table sequentially.",
    "vis.kb.bitmapScan.pros":
      'Solves the "random I/O" problem of standard Index Scans when fetching a moderate amount of data.',
    "vis.kb.bitmapScan.cons":
      "Consumes memory to build the bitmap. Lossy bitmaps can occur if memory is insufficient.",

    "vis.kb.nestLoop.title": "Nested Loop Join",
    "vis.kb.nestLoop.desc":
      "For every row in the outer table, it scans the inner table for matches.",
    "vis.kb.nestLoop.pros":
      "Best for joining small datasets or when the inner table is efficiently indexed.",
    "vis.kb.nestLoop.cons":
      "Performance degrades exponentially (M * N) if tables are large and the inner table lacks an index.",

    "vis.kb.hashJoin.title": "Hash Join",
    "vis.kb.hashJoin.desc":
      "Builds a hash table from the inner table in memory, then probes it with rows from the outer table.",
    "vis.kb.hashJoin.pros":
      "The standard for joining large, unsorted datasets. Usually faster than Nested Loop for bulk data.",
    "vis.kb.hashJoin.cons":
      "High memory usage. If work_mem is exceeded, it spills to disk (temp files), killing performance.",

    "vis.kb.mergeJoin.title": "Merge Join",
    "vis.kb.mergeJoin.desc": "Zips two sorted input streams together.",
    "vis.kb.mergeJoin.pros":
      "Extremely fast and memory-efficient if inputs are already sorted (e.g., by index).",
    "vis.kb.mergeJoin.cons":
      "Requires sorting data first (Sort node) if indices are not available, which is expensive.",

    "vis.kb.agg.title": "Aggregation (Hash/Group)",
    "vis.kb.agg.desc":
      "Performs grouping operations (GROUP BY) or duplicate removal.",
    "vis.kb.agg.pros":
      "HashAggregate is generally faster but uses memory. GroupAggregate requires sorted input but uses less memory.",
    "vis.kb.agg.cons":
      'Watch for "Disk" spills in HashAggregate if memory is insufficient.',

    "vis.kb.sort.title": "Sort",
    "vis.kb.sort.desc":
      "Sorts the dataset in memory (QuickSort) or on disk (MergeSort).",
    "vis.kb.sort.pros":
      "Necessary for ORDER BY, Merge Joins, and some Window Functions.",
    "vis.kb.sort.cons":
      "CPU intensive. Disk-based sorts are significantly slower than memory sorts.",

    "vis.kb.cteScan.title": "CTE Scan (Common Table Expression)",
    "vis.kb.cteScan.desc":
      "Reads results from a temporary result set defined in a WITH clause.",
    "vis.kb.cteScan.pros":
      "Makes SQL readable and modular. Allows reusing the same result set multiple times in a query.",
    "vis.kb.cteScan.cons":
      "Acts as an optimization fence (materialized) in some versions, preventing predicate pushdown.",

    "vis.kb.materialize.title": "Materialize",
    "vis.kb.materialize.desc":
      "Stores the result of a sub-operation in memory (or disk) to allow repeated access.",
    "vis.kb.materialize.pros":
      "Essential for Nested Loops where the inner side is complex, or for Merge Joins.",
    "vis.kb.materialize.cons":
      "Adds startup overhead and consumes memory. High cost if the result set is large.",

    "vis.kb.limit.title": "Limit",
    "vis.kb.limit.desc":
      "Stops processing once a specified number of rows have been returned.",
    "vis.kb.limit.pros":
      'Crucial for pagination and "Top N" queries. Reduces workload significantly if top rows are found quickly.',
    "vis.kb.limit.cons":
      "If the underlying sort or filter is expensive, Limit only helps after the first N rows are computed.",

    "vis.kb.subqueryScan.title": "Subquery Scan",
    "vis.kb.subqueryScan.desc":
      "Reads the output of a subquery as if it were a physical table.",
    "vis.kb.subqueryScan.pros":
      "Enables processing of complex operations (like grouping/window functions) before joining.",
    "vis.kb.subqueryScan.cons":
      'Often implies the optimizer could not "flatten" the query, potentially preventing index usage on underlying tables.',

    // Visualizer Help Guide
    "vis.guide.title": "GaussDB Execution Plan Guide",
    "vis.guide.scans": "Scan Methods",
    "vis.guide.joins": "Join Methods",
    "vis.guide.others": "Other Operators",
    "vis.guide.seqScan": "Seq Scan",
    "vis.guide.seqScanDesc":
      "Full table scan. Reads every row in the table. Usually efficient for small tables or when retrieving a large percentage of rows, but slow for large tables with selective filters.",
    "vis.guide.indexScan": "Index Scan",
    "vis.guide.indexScanDesc":
      "Uses an index to find specific rows. Much faster than Seq Scan for selective queries.",
    "vis.guide.bitmapScan": "Bitmap Heap/Index Scan",
    "vis.guide.bitmapScanDesc":
      "Combines multiple index scans or handles too many non-sequential row fetches efficiently.",
    "vis.guide.nestLoop": "Nested Loop",
    "vis.guide.nestLoopDesc":
      "Joins two tables by looping through every row of the outer table and finding matches in the inner table. Efficient when the outer table is small or the inner table is indexed.",
    "vis.guide.hashJoin": "Hash Join",
    "vis.guide.hashJoinDesc":
      'Loads the candidate rows from the "inner" table into a hash table, then scans the "outer" table to probe for matches. Good for large, unsorted datasets.',
    "vis.guide.mergeJoin": "Merge Join",
    "vis.guide.mergeJoinDesc":
      "Joins two sorted datasets. Very efficient if the input data is already sorted (e.g., by an index).",
    "vis.guide.agg": "Aggregation (Hash/Group)",
    "vis.guide.aggDesc":
      "Operations like GROUP BY or DISTINCT. HashAggregate uses a hash table, while GroupAggregate requires sorted input.",
  },
  zh: {
    // Menu
    "menu.dashboard": "仪表盘",
    "menu.reports": "WDR 报告",
    "menu.comparison": "对比分析",
    "menu.visualizer": "执行计划分析",
    "menu.thresholds": "阈值配置",
    "menu.sqlaudit": "SQL 审计",
    "menu.auditlog": "审计日志",

    // Header
    "header.role": "DBA 管理员",

    // Dashboard
    "dash.cpu": "CPU 使用率",
    "dash.mem": "内存使用率",
    "dash.tps": "TPS",
    "dash.qps": "QPS",
    "dash.health": "实例健康度",
    "dash.trend": "Top 问题趋势 (今日)",
    "dash.hotIssues": "Top 10 热门问题",
    "dash.recentReports": "近期 WDR 报告",
    "dash.view": "查看",
    "dash.details": "详情",
    "dash.healthy": "健康",
    "dash.warning": "警告",
    "dash.critical": "严重",
    "dash.instanceOverview": "实例概览",
    "dash.selectInstance": "选择实例",
    "dash.allInstances": "所有实例",
    "dash.lastReport": "最新报告",
    "dash.activeIssues": "活跃问题",
    "dash.score": "健康分",

    // Reports
    "rep.search": "搜索实例...",
    "rep.upload": "上传报告",
    "rep.importReport": "导入报告",
    "rep.id": "ID",
    "rep.instance": "实例",
    "rep.generated": "生成时间",
    "rep.period": "时段",
    "rep.status": "状态",
    "rep.actions": "操作",
    "rep.loading": "加载中...",
    "rep.uploadTitle": "导入 WDR 报告",
    "rep.instanceName": "实例名称",
    "rep.description": "描述",
    "rep.file": "报告文件",
    "rep.filePath": "文件路径",
    "rep.fileRequired": "文件路径是必需的",
    "rep.instanceRequired": "实例名称是必需的",
    "rep.fileHint": "支持 HTML 或 WDR 文件，最大 50MB",
    "rep.cancel": "取消",
    "rep.submit": "导入报告",
    "rep.import": "导入",
    "rep.uploading": "上传中...",
    "rep.uploadFailed": "上传失败",
    "rep.showing": "显示 {start}-{end} 共 {total}",
    "rep.prev": "上一页",
    "rep.next": "下一页",
    "rep.deleteTitle": "删除报告",
    "rep.deleteConfirm": "确定要删除报告 #{id} 吗？此操作无法撤销。",
    "rep.delete": "删除",
    "rep.viewTitle": "报告详情",
    "rep.tab.summary": "摘要",
    "rep.tab.load": "负载概况",
    "rep.tab.sqlstats": "SQL 统计",
    "rep.tab.objstats": "对象统计",
    "rep.summary.efficiency": "实例效率指标",
    "rep.summary.workload": "负载概况 (每秒)",
    "rep.summary.db": "数据库统计",
    "rep.metric": "指标",
    "rep.perSec": "每秒",
    "rep.perTxn": "每事务",
    "rep.sql.orderBy": "排序方式",
    "rep.sql.elapsed": "执行耗时",
    "rep.sql.cpu": "CPU 时间",
    "rep.sql.calls": "执行次数",
    "rep.sql.reads": "物理读",
    "rep.sql.text": "完整 SQL",
    "rep.sql.visualize": "可视化计划",
    "rep.obj.table": "表统计",
    "rep.obj.index": "索引统计",
    "rep.obj.deadTup": "死元组",
    "rep.obj.liveTup": "活元组",
    "rep.obj.seqScan": "全表扫描",
    "rep.obj.idxScan": "索引扫描",

    // Comparison
    "comp.selected": "已选报告",
    "comp.new": "新建对比",
    "comp.name": "对比名称",
    "comp.desc": "描述",
    "comp.save": "保存",
    "comp.tab.sql": "SQL 统计",
    "comp.tab.wait": "等待事件",
    "comp.tab.obj": "对象统计",
    "comp.tab.sys": "系统指标",
    "comp.fingerprint": "SQL 指纹",
    "comp.r1": "报告 1",
    "comp.r2": "报告 2",
    "comp.change": "变化率",
    "comp.plan": "计划",
    "comp.action": "操作",
    "comp.col.event": "事件名称",
    "comp.col.class": "类别",
    "comp.col.time": "耗时 (ms)",
    "comp.col.object": "对象名称",
    "comp.col.schema": "模式",
    "comp.col.scans": "扫描次数",
    "comp.col.metric": "指标名称",
    "comp.col.val": "数值",
    "comp.col.diff": "差值",
    "comp.col.cpu": "CPU时间",
    "comp.col.io": "IO时间",
    "comp.col.seq": "全表扫描",
    "comp.col.idx": "索引扫描",
    "comp.col.tup": "元组(I/U/D)",
    "comp.overview.title": "指标对比概览",
    "comp.chart.dbTime": "DB Time (秒)",
    "comp.chart.cpu": "CPU 使用率 (%)",
    "comp.chart.io": "IOPS",
    "comp.col.phyRd": "物理读",
    "comp.col.logRd": "逻辑读",
    "comp.col.heapRd": "堆读取",
    "comp.col.heapHit": "堆命中",
    "comp.col.idxRd": "索引读取",
    "comp.col.idxHit": "索引命中",

    // Comparison Summary
    "comp.summary.title": "分析摘要",
    "comp.summary.conclusion": "分析结论",
    "comp.summary.findings": "关键发现",
    "comp.summary.score": "性能评分",
    "comp.status.improved": "优化",
    "comp.status.degraded": "退化",
    "comp.status.stable": "稳定",

    // Thresholds
    "thr.categories": "分类",
    "thr.config": "配置",
    "thr.batchSave": "批量保存",
    "thr.templates": "模板",
    "thr.sysTemplates": "系统模板",
    "thr.customTemplates": "自定义模板",
    "thr.key": "键名",
    "thr.value": "阈值",
    "thr.unit": "单位",
    "thr.range": "推荐范围",
    "thr.action": "操作",

    // SQL Audit
    "audit.all": "全部",
    "audit.pending": "待处理",
    "audit.processing": "处理中",
    "audit.fixed": "已修复",
    "audit.whitelisted": "白名单",
    "audit.id": "ID",
    "audit.severity": "严重级别",
    "audit.type": "类型",
    "audit.target": "目标",
    "audit.foundTime": "发现时间",
    "audit.status": "状态",
    "audit.actions": "操作",
    "audit.optimize": "优化",
    "audit.modalTitle": "SQL 优化诊断",
    "audit.issueId": "问题 ID",
    "audit.targetSql": "目标 SQL",
    "audit.diagnosis": "诊断分析",
    "audit.recommendation": "优化建议",
    "audit.apply": "应用修复",
    "audit.whitelist": "加入白名单",
    "audit.cancel": "取消",

    // Audit Log
    "log.allOps": "所有操作",
    "log.updateThr": "更新阈值",
    "log.export": "导出",
    "log.time": "时间",
    "log.user": "用户",
    "log.op": "操作类型",
    "log.target": "对象",
    "log.result": "结果",

    // Visualizer
    "vis.title": "执行计划可视化",
    "vis.import": "导入",
    "vis.view": "视图",
    "vis.costThreshold": "成本阈值",
    "vis.explain": "执行分析",
    "vis.help": "帮助",
    "vis.sqlEditor": "SQL 编辑器",
    "vis.syntax": "GaussDB 语法",
    "vis.pastePlaceholder": "在此粘贴您的 SQL...",
    "vis.planText": "计划文本",
    "vis.rawExplain": "原始 Explain",
    "vis.noPlan": "未生成计划",
    "vis.visualTree": "可视化树",
    "vis.totalCost": "总成本",
    "vis.analyzing": "正在分析执行计划...",
    "vis.selectSql": "选择 SQL 以查看计划",
    "vis.node.cost": "成本",
    "vis.node.rows": "行数",
    "vis.node.width": "宽度",
    "vis.node.target": "目标对象",
    "vis.node.details": "详情",
    "vis.opt.suggestions": "优化建议",
    "vis.opt.highCost": "检测到高成本连接",
    "vis.opt.highCostDesc":
      "根节点的哈希连接成本为 {cost}，超过了阈值。请检查连接列是否已建立索引。",
    "vis.opt.indexOpp": "索引建议",
    "vis.opt.indexOppDesc": "对 'users' 表进行顺序扫描。建议创建索引。",
    "vis.opt.loadToSee": "加载计划以查看建议",
    "vis.hot.title": "WDR Top SQL",
    "vis.hot.viewFull": "View Full Report",
    "vis.hot.preview": "SQL Preview",
    "vis.hot.time": "Time",
    "vis.hot.cost": "Cost",
    "vis.hot.action": "Action",
    "vis.hot.load": "Load",

    // Knowledge Base
    "vis.kb.title": "算子知识库",
    "vis.kb.search": "搜索算子...",
    "vis.kb.pros": "适用场景 (优点)",
    "vis.kb.cons": "性能风险 (缺点)",

    "vis.kb.seqScan.title": "顺序扫描 (Seq Scan)",
    "vis.kb.seqScan.desc": "从头到尾读取表中的每一行数据。",
    "vis.kb.seqScan.pros":
      "适用于极小的表，或者查询需要获取表中大部分数据（通常>20%）的情况。",
    "vis.kb.seqScan.cons":
      "如果表很大且只需少量行，全表扫描效率极低，会产生大量磁盘 I/O。",

    "vis.kb.idxScan.title": "索引扫描 (Index Scan)",
    "vis.kb.idxScan.desc":
      "遍历 B-Tree 索引找到特定行的物理位置，然后再回表获取数据。",
    "vis.kb.idxScan.pros": "获取少量数据（高选择性）时速度极快。",
    "vis.kb.idxScan.cons":
      "如果需要获取的数据行很多且分散，产生的随机 I/O 会导致性能比顺序扫描更差。",

    "vis.kb.idxOnlyScan.title": "仅索引扫描 (Index Only Scan)",
    "vis.kb.idxOnlyScan.desc":
      "直接从索引中获取所需数据，无需回表读取（覆盖索引）。",
    "vis.kb.idxOnlyScan.pros": "最快的扫描方式，完全避免了随机表读取。",
    "vis.kb.idxOnlyScan.cons":
      "要求查询的所有列都包含在索引中。如果可见性映射（VM）未更新，仍需回表检查可见性。",

    "vis.kb.bitmapScan.title": "位图扫描 (Bitmap Scan)",
    "vis.kb.bitmapScan.desc":
      "先通过索引建立匹配页的位图，排序后按顺序读取表数据。",
    "vis.kb.bitmapScan.pros":
      "解决了普通索引扫描在获取中等量数据时的“随机 I/O”问题。",
    "vis.kb.bitmapScan.cons":
      "构建位图需要消耗内存。如果内存不足，位图可能变得有损（Lossy），导致需重新检查行。",

    "vis.kb.nestLoop.title": "嵌套循环连接 (Nested Loop)",
    "vis.kb.nestLoop.desc":
      "对于外表的每一行，扫描内表以查找匹配项（双重循环）。",
    "vis.kb.nestLoop.pros": "连接小数据集，或内表有高效索引时性能最佳。",
    "vis.kb.nestLoop.cons":
      "如果表很大且内表无索引，性能会呈指数级下降 (M * N)。",

    "vis.kb.hashJoin.title": "哈希连接 (Hash Join)",
    "vis.kb.hashJoin.desc":
      "将“内”表加载到内存哈希表中，然后扫描“外”表进行探测匹配。",
    "vis.kb.hashJoin.pros": "大数据集关联的标准方式，通常比嵌套循环快。",
    "vis.kb.hashJoin.cons":
      "内存消耗大。如果超出 work_mem，会溢出到磁盘（临时文件），严重影响性能。",

    "vis.kb.mergeJoin.title": "归并连接 (Merge Join)",
    "vis.kb.mergeJoin.desc": "像拉链一样合并两个已排序的输入流。",
    "vis.kb.mergeJoin.pros":
      "如果输入数据已经排序（如通过索引），则非常高效且省内存。",
    "vis.kb.mergeJoin.cons": "如果数据未排序，需要先执行 Sort 操作，成本昂贵。",

    "vis.kb.agg.title": "聚合 (Hash/Group)",
    "vis.kb.agg.desc": "执行 GROUP BY 分组或去重操作。",
    "vis.kb.agg.pros":
      "HashAggregate 通常更快但耗内存；GroupAggregate 省内存但需排序。",
    "vis.kb.agg.cons": "HashAggregate 内存不足时会落盘，需关注性能日志。",

    "vis.kb.sort.title": "排序 (Sort)",
    "vis.kb.sort.desc":
      "在内存（QuickSort）或磁盘（MergeSort）中对数据进行排序。",
    "vis.kb.sort.pros": "ORDER BY、归并连接和部分窗口函数所必需的。",
    "vis.kb.sort.cons":
      "CPU 密集型操作。一旦内存不足触发磁盘排序，速度会大幅下降。",

    "vis.kb.cteScan.title": "CTE 扫描 (CTE Scan)",
    "vis.kb.cteScan.desc": "读取 WITH 子句定义的临时结果集。",
    "vis.kb.cteScan.pros":
      "提高 SQL 可读性和模块化。允许在查询中多次复用同一个结果集。",
    "vis.kb.cteScan.cons":
      "在某些版本中会作为优化栅栏（物化），阻止谓词下推，可能导致性能降低。",

    "vis.kb.materialize.title": "物化 (Materialize)",
    "vis.kb.materialize.desc":
      "将子操作的结果存储在内存（或磁盘）中，以便重复访问。",
    "vis.kb.materialize.pros": "对于内表复杂的嵌套循环连接或归并连接是必需的。",
    "vis.kb.materialize.cons":
      "增加启动开销并消耗内存。如果结果集很大，成本会很高。",

    "vis.kb.limit.title": "限制 (Limit)",
    "vis.kb.limit.desc": "一旦返回指定数量的行，就停止处理。",
    "vis.kb.limit.pros":
      "对于分页和“Top N”查询至关重要。如果能快速找到前 N 行，可显著减少负载。",
    "vis.kb.limit.cons":
      "如果底层的排序或过滤很昂贵，Limit 只有在计算出前 N 行后才能发挥作用。",

    "vis.kb.subqueryScan.title": "子查询扫描 (Subquery Scan)",
    "vis.kb.subqueryScan.desc": "将子查询的输出作为物理表进行读取。",
    "vis.kb.subqueryScan.pros":
      "允许在连接之前处理复杂操作（如分组/窗口函数）。",
    "vis.kb.subqueryScan.cons":
      "通常意味着优化器无法“扁平化”查询，可能阻止对底层表使用索引。",

    // Visualizer Help Guide
    "vis.guide.title": "GaussDB 执行计划指南",
    "vis.guide.scans": "扫描方式",
    "vis.guide.joins": "连接方式",
    "vis.guide.others": "其他算子",
    "vis.guide.seqScan": "顺序扫描 (Seq Scan)",
    "vis.guide.seqScanDesc":
      "全表扫描。读取表中的每一行。对于小表或读取大部分数据时效率较高，但对于大表且过滤性强的查询，效率较低。",
    "vis.guide.indexScan": "索引扫描 (Index Scan)",
    "vis.guide.indexScanDesc":
      "使用索引查找特定行。对于选择性高的查询，比全表扫描快得多。",
    "vis.guide.bitmapScan": "位图扫描 (Bitmap Heap/Index Scan)",
    "vis.guide.bitmapScanDesc":
      "结合多个索引扫描，或高效处理大量非连续的行获取。",
    "vis.guide.nestLoop": "嵌套循环 (Nested Loop)",
    "vis.guide.nestLoopDesc":
      "通过遍历外表的每一行并在内表中查找匹配项来连接两个表。当外表较小或内表有索引时效率最高。",
    "vis.guide.hashJoin": "哈希连接 (Hash Join)",
    "vis.guide.hashJoinDesc":
      "将“内”表的候选行加载到哈希表中，然后扫描“外”表以探测匹配项。适用于大型、未排序的数据集。",
    "vis.guide.mergeJoin": "归并连接 (Merge Join)",
    "vis.guide.mergeJoinDesc":
      "连接两个已排序的数据集。如果输入数据已经排序（例如通过索引），则非常高效。",
    "vis.guide.agg": "聚合 (Hash/Group)",
    "vis.guide.aggDesc":
      "GROUP BY 或 DISTINCT 等操作。HashAggregate 使用哈希表，而 GroupAggregate 需要输入已排序。",
  },
};

interface I18nContextProps {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string, params?: Record<string, any>) => string;
}

const I18nContext = createContext<I18nContextProps | undefined>(undefined);

export const I18nProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  const [language, setLanguage] = useState<Language>(() => {
    return (localStorage.getItem("app_language") as Language) || "zh";
  });

  useEffect(() => {
    localStorage.setItem("app_language", language);
  }, [language]);

  const t = (key: string, params?: Record<string, any>) => {
    let text =
      translations[language][key as keyof (typeof translations)["en"]] || key;
    if (params) {
      Object.entries(params).forEach(([k, v]) => {
        text = text.replace(`{${k}}`, String(v));
      });
    }
    return text;
  };

  return (
    <I18nContext.Provider value={{ language, setLanguage, t }}>
      {children}
    </I18nContext.Provider>
  );
};

export const useI18n = () => {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider");
  }
  return context;
};
