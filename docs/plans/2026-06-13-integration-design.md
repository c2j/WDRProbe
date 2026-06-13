# WDRProbe 集成设计规划：多模式 + 外部引擎嵌入方案

> **目标读者**：对 WDRProbe 代码库有基本了解的开发者。
> **前置阅读**：`AGENTS.md`、`docs/desktop-IPC.md`

**目标：** 
1. 将 ogexplain-analyzer 和 metamorphosis 作为外部引擎嵌入 WDRProbe
2. 从纯 Tauri 桌面应用重构为 **CLI + TUI + Desktop 三模架构**，共享 `wdrprobe-core`
3. 使 WDRProbe 从 "WDR 报告查看器" 升级为 "GaussDB 性能诊断与 SQL 优化一站式工具"

**设计原则：**
1. **双解析器共存，渐进替换**——不立即移除 `sql_parser.rs`，待新架构稳定后再退役
2. **Git Dependency，非 Workspace 统一**——外部项目通过 `git = "..."` 引入，保持独立迭代
3. **适配层隔离**——外部 AST 类型通过转换层映射到 WDRProbe 自有类型
4. **Core-First，多前端**——提取 `wdrprobe-core` 后，Desktop/TUI/CLI 均作为其消费者
5. **增量交付**——每阶段产出可独立验证的用户价值

---

## 一、目标架构（三模 + 双引擎）

```
                    ┌──────────────────────────────────────────┐
                    │            wdrprobe-core                  │
                    │  (纯 Rust 库，零 UI 依赖)                  │
                    │                                          │
                    │  database/  parsers/  models/            │
                    │  utils/     progress/                    │
                    │  (WDR 报告解析、SQLite 存储、对比算法)      │
                    └──────┬──────────┬──────────┬─────────────┘
                           │          │          │
              ┌────────────┘          │          └────────────┐
              ▼                       ▼                       ▼
    ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
    │ wdrprobe-desktop │  │  wdrprobe-cli    │  │  wdrprobe-tui    │
    │ (Tauri v1)       │  │  (clap)          │  │  (ratatui)       │
    │                  │  │                  │  │                  │
    │ React 前端       │  │ import/analyze/  │  │ Dashboard /      │
    │ 38 IPC 命令      │  │ audit/export     │  │ Reports /        │
    │ adapters/        │  │ 子命令           │  │ Plan Tree /      │
    │   ├─ ogexplain   │  │                  │  │ Comparison       │
    │   ├─ metamorphosis│ │                  │  │                  │
    │   └─ schema      │  │                  │  │                  │
    └────────┬─────────┘  └──────────────────┘  └──────────────────┘
             │
    ┌────────┴──────────────────────────────┐
    │           外部引擎 (git deps)           │
    │                                        │
    │  ogexplain-core    metamorphosis-core  │
    │  parse()           RewriteEngine       │
    │  analyze()         RuleRegistry        │
    │  heatmap()         4 built-in rules    │
    │  waterfall()                           │
    │        │                    │          │
    │        └────────┬───────────┘          │
    │                 ▼                      │
    │          ogsql-parser                  │
    │          (AST / Parser / Formatter)    │
    └────────────────────────────────────────┘
```

**三模定位：**

| 模式 | 目标用户 | 使用场景 |
|------|---------|---------|
| **Desktop** (Tauri) | DBA / 开发者 | 交互式 WDR 报告浏览、执行计划可视化、诊断仪表盘 |
| **CLI** (clap) | CI/CD / 脚本 | 批量 WDR 导入、自动化审计、报告导出 |
| **TUI** (ratatui) | 终端 DBA | SSH 远程服务器上的快速诊断、无 GUI 环境 |

**关键边界：**
- `wdrprobe-core` 被三个前端共享，零 UI 依赖
- `adapters/` 保留在 Desktop crate 中——依赖外部引擎，不进入 core
- `ogexplain-core` 同时服务 Desktop（诊断页面）和 CLI/TUI（诊断输出）
- `sql_parser.rs` **保留并存**，待 Phase 7 才退役

---

## 二、分阶段实施计划

### 全景路线图

```
轨道 A：外部引擎集成              轨道 B：多模式架构
                                  
Phase 1: ogexplain-core 嵌入       Phase 3: wdrprobe-core 提取
Phase 2: 前端诊断页面                    ↓
         ↓                        Phase 4: CLI 模式
Phase 6: metamorphosis 接入       Phase 5: TUI 模式
         ↓                              ↓
Phase 7: sql_parser.rs 退役       共享 core，三模就绪
```

**并行策略：** Phase 2（前端）和 Phase 3（core 提取）可由不同开发者并行推进。Phase 4-5（CLI/TUI）在 Phase 3 完成后启动。

---

### Phase 0：基础设施准备（已完成 ✅，2026-06-13）

- ✅ 清理根目录垃圾文件（空 `package-lock.json`、散落测试脚本 → `experiments/`）
- ✅ 添加 `LICENSE-MIT` 和 `LICENSE-APACHE`
- ✅ 添加 CI PR 门禁（`cargo test` + `cargo clippy` + `tsc --noEmit`）

---

### 轨道 A：外部引擎集成

### Phase 1：ogexplain-core 嵌入（P0，3-5 天）

**目标：** WDRProbe Desktop 获得 GaussDB 执行计划 25 条诊断规则能力。

| 任务 | 文件 | 工作量 |
|------|------|--------|
| 1.1 添加外部依赖 | `Desktop/src-tauri/Cargo.toml` | 0.5h |
| 1.2 创建适配层 | `src/adapters/ogexplain_adapter.rs` (NEW) | 4h |
| 1.3 新增 IPC 命令（6 个） | `src/commands/execution_plan.rs` | 4h |
| 1.4 注册新命令 | `src/main.rs` | 0.5h |
| 1.5 TypeScript API 封装 | `frontend/services/apiService.ts` | 2h |
| 1.6 TypeScript 类型定义 | `frontend/types.ts` | 2h |
| 1.7 后端单元测试 | `src-tauri/tests/ogexplain_integration_test.rs` (NEW) | 3h |
| 1.8 i18n 翻译 key 补充 | `frontend/context/I18nContext.tsx` | 3h |

#### 1.1 依赖声明

```toml
# Desktop/src-tauri/Cargo.toml 新增
[dependencies]
ogexplain-core = { git = "https://github.com/c2j/ogexplain-analyzer.git", tag = "v0.2.0" }

# ogsql-parser 作为传递依赖自动引入（ogexplain-core 依赖它）
# 如需直接使用 ogsql-parser（如未来做 SQL AST 分析），可显式声明：
# ogsql-parser = { git = "https://github.com/c2j/ogsql-parser.git", tag = "v0.6.20" }
```

#### 1.2 适配层设计 (`src/adapters/ogexplain_adapter.rs`)

```rust
//! ogexplain-core → WDRProbe 类型转换适配层
//!
//! 职责：
//! 1. ogexplain_core::model::PlanNode → WDRProbe ExecutionPlanNode
//! 2. ogexplain_core::analyzer::Finding → 前端友好格式
//! 3. 热力图/瀑布图数据映射

use ogexplain_core::model::{ExplainPlan, PlanNode};
use ogexplain_core::analyzer::report::{DiagnosticReport, Finding, Severity};
use crate::models::execution_plan::{
    ExecutionPlanNode, PlanNodeDetails, PlanMetadata,
    ExecutionPlanResponse, PlanIssue,
};

/// 将 ogexplain-core 的 PlanNode 树转换为 WDRProbe 的 ExecutionPlanNode 树
pub fn convert_plan_node(node: &PlanNode) -> ExecutionPlanNode {
    ExecutionPlanNode {
        operation: node.node_type.to_string(),
        cost: node.estimated.as_ref()
            .map(|e| e.total_cost)
            .unwrap_or(0.0),
        rows: node.estimated.as_ref()
            .map(|e| e.plan_rows as u64)
            .unwrap_or(0),
        actual_rows: node.actual.as_ref().map(|a| a.actual_rows as u64),
        actual_time: node.actual.as_ref().map(|a| a.actual_time),
        width: node.estimated.as_ref().map(|e| e.plan_width as u32),
        children: node.children.iter().map(convert_plan_node).collect(),
        node_details: convert_node_details(node),
        warnings: Vec::new(),   // 由诊断层填充
        suggestions: Vec::new(), // 由诊断层填充
    }
}

/// 转换诊断发现 → 前端 PlanIssue
pub fn convert_findings_to_issues(
    findings: &[Finding],
    node_map: &HashMap<usize, String>, // line_number → node_uid
) -> Vec<PlanIssue> {
    findings.iter().map(|f| PlanIssue {
        rule_id: f.rule_id.clone(),
        title: f.title.clone(),
        severity: convert_severity(&f.severity),
        issue_type: format!("{:?}", f.category),
        description: f.detail.clone(),
        suggestion: f.suggestion.clone(),
        node_uids: f.node_line
            .and_then(|line| node_map.get(&line).cloned())
            .map(|uid| vec![uid])
            .unwrap_or_default(),
        impact_score: None,     // Phase 2 从 metamorphosis 获取
        confidence: None,       // Phase 2 从 metamorphosis 获取
    }).collect()
}

// 私有辅助函数
fn convert_node_details(node: &PlanNode) -> PlanNodeDetails { /* ... */ }
fn convert_severity(s: &Severity) -> String { /* Critical → "critical" */ }
```

#### 1.3 新增 Tauri 命令（6 个）

在 `src/commands/execution_plan.rs` 中新增（不修改现有命令）：

```rust
use crate::adapters::ogexplain_adapter;

/// 使用 ogexplain-core 解析执行计划文本
#[tauri::command]
pub async fn parse_explain_with_ogexplain(
    plan_text: String,
) -> Result<ExecutionPlanNode, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(ogexplain_adapter::convert_plan_node(&plan.root))
}

/// 诊断执行计划（25 条规则）
#[tauri::command]
pub async fn diagnose_explain_plan(
    plan_text: String,
) -> Result<DiagnosticReportResponse, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    let report = ogexplain_core::analyze(&plan);
    Ok(ogexplain_adapter::convert_diagnostic_report(&report, &plan))
}

/// 生成热力图数据（需 EXPLAIN ANALYZE）
#[tauri::command]
pub async fn get_explain_heatmap(
    plan_text: String,
) -> Result<Option<HeatmapData>, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(ogexplain_core::heatmap(&plan)
        .map(|h| ogexplain_adapter::convert_heatmap(&h)))
}

/// 生成资源瀑布图数据（需 EXPLAIN ANALYZE）
#[tauri::command]
pub async fn get_explain_waterfall(
    plan_text: String,
) -> Result<Option<WaterfallData>, String> {
    let plan = ogexplain_core::parse(&plan_text)
        .map_err(|e| format!("Parse error: {}", e))?;
    Ok(ogexplain_core::waterfall(&plan)
        .map(|w| ogexplain_adapter::convert_waterfall(&w)))
}

/// SQL 复杂度评分
#[tauri::command]
pub async fn score_sql_complexity(
    sql: String,
) -> Result<ComplexityScoreResponse, String> {
    // ogsql-complexity 暂不直接依赖（需 ogexplain-analyzer 暴露或单独引入）
    // Phase 1 先跳过，Phase 2 引入
    Err("Not implemented in Phase 1".to_string())
}

/// 获取诊断规则列表（供前端展示规则目录）
#[tauri::command]
pub async fn list_diagnostic_rules() -> Result<Vec<RuleInfo>, String> {
    // 静态返回 25 条规则元数据
    Ok(ogexplain_adapter::get_rule_catalog())
}
```

#### 1.4 注册命令（`src/main.rs`）

```rust
tauri::generate_handler![
    // ... 现有 38 个命令 ...
    
    // Phase 1 新增
    execution_plan::parse_explain_with_ogexplain,
    execution_plan::diagnose_explain_plan,
    execution_plan::get_explain_heatmap,
    execution_plan::get_explain_waterfall,
    execution_plan::score_sql_complexity,
    execution_plan::list_diagnostic_rules,
]
```

#### 1.5-1.6 TypeScript 封装

```typescript
// frontend/services/apiService.ts 新增
export const ApiService = {
  // ... 现有方法 ...

  // Phase 1: ogexplain-analyzer 集成
  parseExplainWithOgexplain: async (planText: string): Promise<ExecutionPlanNode> => {
    return invoke('parse_explain_with_ogexplain', { planText });
  },

  diagnoseExplainPlan: async (planText: string): Promise<DiagnosticReportResponse> => {
    return invoke('diagnose_explain_plan', { planText });
  },

  getExplainHeatmap: async (planText: string): Promise<HeatmapData | null> => {
    return invoke('get_explain_heatmap', { planText });
  },

  getExplainWaterfall: async (planText: string): Promise<WaterfallData | null> => {
    return invoke('get_explain_waterfall', { planText });
  },

  listDiagnosticRules: async (): Promise<RuleInfo[]> => {
    return invoke('list_diagnostic_rules');
  },
};
```

```typescript
// frontend/types.ts 新增类型
export interface DiagnosticReportResponse {
  findings: PlanIssue[];
  stats: {
    totalNodes: number;
    totalFindings: number;
    criticalCount: number;
    warningCount: number;
    infoCount: number;
  };
}

export interface RuleInfo {
  ruleId: string;
  category: string;
  title: string;
  description: string;
  severity: 'Critical' | 'Warning' | 'Info';
}

export interface HeatmapData {
  nodes: Array<{
    nodeUid: string;
    operation: string;
    estimatedCost: number;
    actualCost: number;
    qError: number;
    severity: 'Negligible' | 'Minor' | 'Moderate' | 'Severe' | 'Extreme';
  }>;
  summary: {
    maxQError: number;
    avgQError: number;
    nodesWithDeviation: number;
  };
}

export interface WaterfallData {
  nodes: Array<{
    nodeUid: string;
    operation: string;
    cpuTime: number;
    memoryKb: number;
    percentage: number;
  }>;
  bottlenecks: {
    cpuBottlenecks: string[];
    memoryBottlenecks: string[];
  };
}
```

---

### Phase 2：前端诊断页面 + 可视化（P0，4-6 天）

**目标：** 前端新增执行计划诊断面板，在 PlanVisualizer 上渲染诊断图标。

| 任务 | 文件 | 工作量 |
|------|------|--------|
| 2.1 PlanVisualizer 增加诊断模式 | `frontend/pages/PlanVisualizer.tsx` | 4h |
| 2.2 新增 Heatmap 页面 | `frontend/pages/PlanHeatmap.tsx` (NEW) | 6h |
| 2.3 新增 Waterfall 页面 | `frontend/pages/PlanWaterfall.tsx` (NEW) | 5h |
| 2.4 新增诊断规则目录页 | `frontend/pages/DiagnosticRules.tsx` (NEW) | 3h |
| 2.5 路由注册 | `frontend/App.tsx` | 1h |
| 2.6 i18n 翻译补充 | `frontend/context/I18nContext.tsx` | 3h |
| 2.7 前端集成测试 | Playwright 端到端测试 | 4h |

#### 2.1 PlanVisualizer 诊断模式

在现有执行计划树节点上叠加诊断图标：

```
现有节点渲染：
  Seq Scan on t_order  (cost=0.00..1500.00 rows=100000)

Phase 2 叠加：
  !! Seq Scan on t_order  (cost=0.00..1500.00 rows=100000)
  └─ SCAN-001: Large table full scan → CREATE INDEX ON t_order(...)
```

**交互：**
- 节点旁显示 `!!`（Critical）/ `!`（Warning）/ `*`（Info）图标
- 悬停显示诊断详情 tooltip
- 侧边栏切换 "全部发现" / "仅严重" 过滤

---

### Phase 3：WDRProbe 内部 Core 提取（P1，3-4 天）

**目标：** 提取 `wdrprobe-core` crate，为 CLI/TUI 双模打基础。此阶段**不涉及外部 crate 集成变更**。

| 任务 | 工作量 |
|------|--------|
| 3.1 创建 `crates/wdrprobe-core/` | 0.5h |
| 3.2 移动 `database/` → core | 1h |
| 3.3 移动 `parsers/` → core | 1h |
| 3.4 移动 `models/` → core | 0.5h |
| 3.5 移动 `utils/` + `progress/` → core | 0.5h |
| 3.6 更新 Desktop Cargo.toml 依赖 | 0.5h |
| 3.7 更新所有 `crate::` import → `wdrprobe_core::` | 2h |
| 3.8 验证 `cargo test --workspace` 全绿 | 1h |

**详细步骤参见** `docs/plans/2026-06-13-core-extraction.md`（待创建）。

**注意：** `adapters/` 目录保留在 `Desktop/src-tauri/src/` 中，不移入 core——因为它们依赖外部 crate（`ogexplain-core`），属于 Tauri 集成层而非纯 WDR 核心逻辑。

---

### 轨道 B：多模式架构

### Phase 4：CLI 模式（P1，2-3 天）

**前置条件：** Phase 3（`wdrprobe-core` 已提取）

**目标：** 创建 `wdrprobe-cli` 二进制，可通过命令行完成 WDR 导入、分析、审计、导出。

**使用场景示例：**
```bash
# 导入 WDR 报告
wdrprobe import --db ./wdrprobe.db opengauss_v1.html

# 分析执行计划
wdrprobe analyze --plan "EXPLAIN SELECT ..."

# 运行 SQL 审计
wdrprobe audit --db ./wdrprobe.db --report-id 42

# 导出对比报告
wdrprobe export --db ./wdrprobe.db --comparison-id 7 --format csv

# 诊断执行计划（调用 ogexplain-core）
wdrprobe diagnose --plan-file explain_output.txt --lang zh-CN
```

**crate 结构：**
```
crates/wdrprobe-cli/
├── Cargo.toml
└── src/
    ├── main.rs          # clap 参数解析 + 子命令路由
    ├── commands/
    │   ├── mod.rs
    │   ├── import.rs    # parse_complete_wdr_report() + DB 存储
    │   ├── analyze.rs   # get_wdr_report_detail() → 终端输出
    │   ├── audit.rs     # run_sql_audit() → 终端输出
    │   ├── export.rs    # export_wdr_report() → 文件
    │   └── diagnose.rs  # ogexplain_core::analyze() → 终端输出
    └── output.rs        # 终端格式化输出（text/json/table）
```

**依赖：**
```toml
[dependencies]
wdrprobe-core = { path = "../wdrprobe-core" }
ogexplain-core = { git = "https://github.com/c2j/ogexplain-analyzer.git", tag = "v0.2.0" }
clap = { version = "4", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

| 任务 | 工作量 |
|------|--------|
| 4.1 创建 `crates/wdrprobe-cli/` + Cargo.toml | 0.5h |
| 4.2 实现 `main.rs`（clap 子命令结构） | 2h |
| 4.3 实现 `commands/import.rs`（调用 core 的 parser + DB） | 2h |
| 4.4 实现 `commands/analyze.rs`（查询 + 格式化输出） | 1.5h |
| 4.5 实现 `commands/audit.rs`（调用检测规则） | 1.5h |
| 4.6 实现 `commands/export.rs`（CSV/JSON 输出） | 1h |
| 4.7 实现 `commands/diagnose.rs`（调用 ogexplain-core） | 1.5h |
| 4.8 实现 `output.rs`（text/JSON/table 格式化） | 2h |
| 4.9 验证：`cargo run -p wdrprobe-cli -- import example/opengauss_v1.html` | 1h |

---

### Phase 5：TUI 模式（P2，5-7 天）

**前置条件：** Phase 3（core 已提取），Phase 4（CLI 完成，验证了 core API）

**目标：** 创建 `wdrprobe-tui` 二进制，在终端中提供交互式 WDR 报告浏览、执行计划可视化、诊断仪表盘。

**crate 结构：**
```
crates/wdrprobe-tui/
├── Cargo.toml
└── src/
    ├── main.rs          # 入口：初始化 DB → 启动 TUI
    ├── app.rs           # TUI 状态机（当前页面、焦点、选中项）
    ├── ui/
    │   ├── mod.rs
    │   ├── dashboard.rs # 效率仪表盘 + 负载概况
    │   ├── reports.rs   # 报告列表（表格 + 分页）
    │   ├── report_detail.rs # 报告详情（多面板）
    │   ├── plan_view.rs # 执行计划树（递归渲染 + 诊断图标）
    │   ├── comparison.rs # 对比视图（并排）
    │   └── audit.rs     # SQL 审计发现列表
    ├── components/
    │   ├── mod.rs
    │   ├── table.rs     # 可排序/分页表格组件
    │   ├── tree.rs      # 递归树渲染组件
    │   └── gauge.rs     # 仪表盘/进度条组件
    └── theme.rs         # 配色方案
```

**依赖：**
```toml
[dependencies]
wdrprobe-core = { path = "../wdrprobe-core" }
ogexplain-core = { git = "https://github.com/c2j/ogexplain-analyzer.git", tag = "v0.2.0" }
ratatui = "0.30"
crossterm = "0.28"
tui-tree-widget = "0.24"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
```

**键盘快捷键设计（参考 ratatui 惯例 + vim 风格）：**

| 全局 | 功能 |
|------|------|
| `Tab` / `Shift+Tab` | 切换面板焦点 |
| `q` | 退出 |
| `?` | 帮助 |

| 报表视图 | 功能 |
|----------|------|
| `j`/`k` 或 `↑`/`↓` | 上下移动 |
| `Enter` | 查看报告详情 |
| `d` | 删除报告 |
| `/` | 搜索/过滤 |

| 执行计划树 | 功能 |
|------------|------|
| `j`/`k` | 节点间移动 |
| `Enter` | 展开/折叠节点 |
| `E` | 全部展开 |
| `W` | 全部折叠 |
| `r` | 切换原始 EXPLAIN 文本 |
| `d` | 显示节点诊断详情 |

| 任务 | 工作量 |
|------|--------|
| 5.1 创建 `crates/wdrprobe-tui/` + Cargo.toml | 0.5h |
| 5.2 实现 `app.rs`（状态机 + 事件循环） | 3h |
| 5.3 实现 `components/table.rs`（可排序表格） | 3h |
| 5.4 实现 `components/tree.rs`（递归树渲染） | 4h |
| 5.5 实现 `ui/dashboard.rs`（仪表盘视图） | 4h |
| 5.6 实现 `ui/reports.rs`（报告列表） | 3h |
| 5.7 实现 `ui/report_detail.rs`（报告详情多面板） | 5h |
| 5.8 实现 `ui/plan_view.rs`（执行计划树 + 诊断图标） | 6h |
| 5.9 实现 `ui/comparison.rs`（并排对比） | 4h |
| 5.10 实现 `ui/audit.rs`（审计发现列表） | 3h |
| 5.11 实现 `theme.rs`（配色方案） | 1h |

**参考项目：** `sqlv`（SQLite 浏览器 TUI）、`tursotui`（多标签数据库浏览）、ogexplain-analyzer 自身的 `ogexplain-tui`

---

### 轨道 A（续）：外部引擎集成

### Phase 6：metamorphosis 接入（P2，3-5 天）

**前置条件：** Phase 1（ogexplain-core 已嵌入），Phase 3（core 已提取，可选）

**目标：** 在 Desktop SQL Audit + CLI audit 命令中增加 "SQL 改写建议" 功能。

| 任务 | 文件 | 工作量 |
|------|------|--------|
| 6.1 添加 metamorphosis 依赖 | `Desktop/src-tauri/Cargo.toml` | 0.5h |
| 6.2 创建适配层 | `src/adapters/metamorphosis_adapter.rs` (NEW) | 4h |
| 6.3 实现 Schema 提取 | `src/adapters/schema_extractor.rs` (NEW) | 4h |
| 6.4 新增改写命令 | `src/commands/audit.rs` (Desktop) + `crates/wdrprobe-cli/src/commands/audit.rs` | 2h |
| 6.5 TypeScript 封装 | `frontend/services/apiService.ts` | 1h |
| 6.6 前端 diff 视图 | `frontend/pages/SqlAudit.tsx` | 6h |

#### 6.2 集成策略：字符串模式（Option C）

metamorphosis 的 `RewriteRule` trait 直接操作 `ogsql_parser::ast::Statement`，WDRProbe 不引入 SQL AST。采用字符串进出模式：

```
WDRProbe SQL 文本
    │
    ▼
ogsql_parser::Parser::parse_sql(sql_text)
    │
    ▼
ogsql_parser::ast::Statement (AST)
    │
    ▼
metamorphosis_core::RewriteEngine::rewrite(ctx, stmts)
    │
    ▼
ogsql_parser::SqlFormatter::format(&rewritten_ast)
    │
    ▼
改写后 SQL 文本 → 前端 diff 视图展示
```

```rust
// src/adapters/metamorphosis_adapter.rs

use metamorphosis_core::{
    RewriteEngine, RuleRegistry, RewriteContext, RewriteConfig,
};
use metamorphosis_rules::builtin_rules;
use ogsql_parser::Parser;

pub struct MetamorphosisAdapter {
    engine: RewriteEngine,
}

impl MetamorphosisAdapter {
    pub fn new() -> Self {
        let registry = RuleRegistry::new(builtin_rules());
        Self { engine: RewriteEngine::new(registry) }
    }

    /// 重写 SQL（Safe 规则自动应用，Conditional 需 Schema 确认）
    pub fn rewrite(
        &self,
        sql: &str,
        schema_json: Option<&str>,
    ) -> Result<RewriteOutput, String> {
        let schema = schema_json
            .map(|s| serde_json::from_str(s).map_err(|e| e.to_string()))
            .transpose()?;

        let (stmts, errors) = Parser::parse_sql(sql);
        if !errors.is_empty() {
            return Err(format!("Parse errors: {:?}", errors));
        }

        let config = RewriteConfig::default();
        let ctx = RewriteContext {
            version: None,
            schema: schema.as_ref(),
            config: &config,
            source_file: None,
            known_variables: None,
        };

        let result = self.engine.rewrite(&ctx, stmts);
        // 将改写后的 AST 格式化回 SQL 文本
        let rewritten_sql = /* ogsql_parser::SqlFormatter::format(&result.statements) */;

        Ok(RewriteOutput {
            original_sql: sql.to_string(),
            rewritten_sql,
            suggestions: result.suggestions.into_iter().map(Into::into).collect(),
            changed: result.changed,
        })
    }
}
```

#### 6.3 Schema 提取器

metamorphosis 的 `eliminate-select-star` 和 `detect-duplicate-eq-keys` 规则需要 Schema 信息。从 WDRProbe 现有数据中提取：

```rust
// src/adapters/schema_extractor.rs

/// 从 WDRProbe 的 object_stats 表提取 Schema 信息
/// 输出 metamorphosis 所需的 SchemaMap 格式：
///   { "table_name": { "column_name": "data_type" } }
pub fn extract_schema_from_wdr(
    pool: &DatabasePool,
    report_id: i64,
) -> Result<SchemaMap, String> {
    // 1. 从 object_stats 获取所有表/索引名
    let objects = DatabaseOperations::get_object_stats(pool, report_id)?;
    // 2. 从 top_sqls 的 SQL 文本中解析 DDL（如 CREATE TABLE）
    // 3. 或从 WDR 报告的配置信息中提取
    // 4. 构建 SchemaMap
    todo!("Schema extraction from WDR report data")
}

/// 手动 Schema 输入（JSON 格式）
pub fn parse_schema_json(json: &str) -> Result<SchemaMap, String> {
    serde_json::from_str(json).map_err(|e| e.to_string())
}
```

#### 6.4 新增命令（Desktop + CLI）

```rust
// src/commands/audit.rs 新增

/// 改写 SQL（应用 Safe 规则）
#[tauri::command]
pub async fn rewrite_sql(
    pool: tauri::State<'_, DatabasePool>,
    sql: String,
    report_id: Option<i64>,
    schema_json: Option<String>,
) -> Result<RewriteResponse, String> {
    let schema = if let Some(json) = schema_json {
        Some(json)
    } else if let Some(rid) = report_id {
        let schema_map = schema_extractor::extract_schema_from_wdr(
            pool.inner(), rid
        )?;
        Some(serde_json::to_string(&schema_map).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let adapter = metamorphosis_adapter::MetamorphosisAdapter::new();
    adapter.rewrite(&sql, schema.as_deref())
}

/// 获取改写规则列表
#[tauri::command]
pub async fn list_rewrite_rules() -> Result<Vec<RewriteRuleInfo>, String> {
    // 静态返回 4 条内置规则元数据
    Ok(vec![
        RewriteRuleInfo {
            id: "eliminate-select-star".into(),
            category: "Semantic".into(),
            safety: "Safe".into(),
            description: "SELECT * → explicit column list".into(),
        },
        // ... 其余 3 条规则
    ])
}
```

---

### Phase 7：sql_parser.rs 退役（P3，1-2 天）

**前置条件：**
- Phase 1-4 全部完成
- 所有前端执行计划功能已切换到 ogexplain-core 解析器
- 前端不再依赖 `parse_execution_plan_json/text/tabular` 旧命令
- 2 周以上生产环境无回归

**退役步骤：**

| 步骤 | 操作 |
|------|------|
| 7.1 | 标记旧命令为 `#[deprecated]`，返回结果同时调用新旧解析器，diff 对比 |
| 7.2 | 2 周后，将旧命令改为仅调用新解析器，移除 diff 逻辑 |
| 7.3 | 2 周后，移除旧命令和 `sql_parser.rs` |
| 7.4 | 移除 `Desktop/src-tauri/Cargo.toml` 中不再需要的 `nom` 依赖 |

---

## 三、依赖关系图（最终态）

```
WDRProbe Workspace
│
├── wdrprobe-core (纯 Rust 库)
│   ├── database/          (rusqlite + r2d2)
│   ├── parsers/
│   │   ├── wdr_parser.rs          (WDR HTML 解析 - 保留)
│   │   ├── complete_wdr_parser.rs (保留)
│   │   └── sql_parser.rs          (Phase 7 退役)
│   ├── models/            (serde)
│   ├── utils/             (thiserror)
│   └── progress/          (chrono)
│
├── wdrprobe-desktop (Tauri v1)
│   ├── adapters/
│   │   ├── ogexplain_adapter.rs   → ogexplain-core (git)
│   │   ├── metamorphosis_adapter.rs → metamorphosis-core (git)
│   │   └── schema_extractor.rs
│   ├── commands/          (IPC 薄包装)
│   └── frontend/          (React 18 + TS 5)
│       ├── PlanVisualizer.tsx
│       ├── PlanHeatmap.tsx      (Phase 2)
│       ├── PlanWaterfall.tsx    (Phase 2)
│       └── DiagnosticRules.tsx   (Phase 2)
│
├── wdrprobe-cli (clap)           (Phase 4)
│   └── commands/
│       ├── import.rs      → wdrprobe-core
│       ├── analyze.rs     → wdrprobe-core
│       ├── audit.rs       → wdrprobe-core
│       ├── export.rs      → wdrprobe-core
│       └── diagnose.rs    → ogexplain-core (git)
│
├── wdrprobe-tui (ratatui)        (Phase 5)
│   └── ui/
│       ├── dashboard.rs   → wdrprobe-core
│       ├── reports.rs     → wdrprobe-core
│       ├── plan_view.rs   → wdrprobe-core + ogexplain-core
│       └── audit.rs       → wdrprobe-core
│
├── ogexplain-core (git: c2j/ogexplain-analyzer)
│   └── ogsql-parser (git: c2j/ogsql-parser) [传递依赖]
│
└── metamorphosis-core (git: c2j/metamorphosis)
    └── ogsql-parser (git: c2j/ogsql-parser) [传递依赖]
```

**依赖方向（核心原则）：**
- `wdrprobe-core` ← 所有三个前端（desktop/cli/tui）都依赖它
- `ogexplain-core` ← desktop + cli + tui 都可依赖它（诊断能力共享）
- `metamorphosis-core` ← desktop + cli 依赖它（改写能力）
- `wdrprobe-core` 不依赖任何外部引擎——保持纯 WDR 逻辑

**注意：** `ogsql-parser` 被两个外部 crate 依赖。Cargo 会解析到同一版本（需确保两边 git tag 兼容）。如有版本冲突，在 Workspace 根 `Cargo.toml` 中用 `[patch]` 统一。

---

## 四、文件变更清单

| 阶段 | 新建文件 | 修改文件 | 删除文件 |
|------|---------|---------|---------|
| Phase 1 | `src/adapters/mod.rs` | `Desktop/src-tauri/Cargo.toml` | — |
| | `src/adapters/ogexplain_adapter.rs` | `src/commands/execution_plan.rs` | |
| | `tests/ogexplain_integration_test.rs` | `src/main.rs` | |
| | | `frontend/services/apiService.ts` | |
| | | `frontend/types.ts` | |
| | | `frontend/context/I18nContext.tsx` | |
| Phase 2 | `frontend/pages/PlanHeatmap.tsx` | `frontend/pages/PlanVisualizer.tsx` | — |
| | `frontend/pages/PlanWaterfall.tsx` | `frontend/App.tsx` | |
| | `frontend/pages/DiagnosticRules.tsx` | | |
| Phase 3 | `crates/wdrprobe-core/` (多个文件) | 全部 `crate::` → `wdrprobe_core::` import | — |
| | `Cargo.toml` (workspace root) | `Desktop/src-tauri/Cargo.toml` | |
| Phase 4 | `crates/wdrprobe-cli/` (10+ 文件) | `Cargo.toml` (workspace members) | — |
| Phase 5 | `crates/wdrprobe-tui/` (15+ 文件) | `Cargo.toml` (workspace members) | — |
| Phase 6 | `src/adapters/metamorphosis_adapter.rs` | `src/commands/audit.rs` | — |
| | `src/adapters/schema_extractor.rs` | `frontend/pages/SqlAudit.tsx` | |
| | | `crates/wdrprobe-cli/src/commands/audit.rs` | |
| Phase 7 | — | `src/commands/execution_plan.rs` | `src/parsers/sql_parser.rs` |
| | | `Cargo.toml` (移除 nom) | |

---

## 五、风险评估与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| ogsql-parser 解析 GaussDB 特有语法失败 | 中 | 高 | 保持双解析器共存，fallback 到旧 parser |
| ogexplain-core 不识别 GaussDB 特有算子（SMP、Vector） | 中 | 中 | ogexplain-core 已支持 80+ NodeType，包含 OG 算子；不足则提 PR |
| 外部 crate git 依赖导致离线构建失败 | 低 | 中 | CI 配置网络访问；或 vendoring（`cargo vendor`） |
| 适配层性能瓶颈（大执行计划 10MB+） | 低 | 中 | Rust 层做异步解析，前端加进度条 |
| metamorphosis Schema 信息不足 | 高 | 低 | Manual 级别规则降级为建议，不自动执行 |
| i18n 冲突（rust-i18n vs React Context） | 低 | 低 | 适配层剥离 rust-i18n 字符串，直接使用原始 key |

---

## 六、验证标准

每个 Phase 完成后：

| 阶段 | 验证命令 | 预期结果 |
|------|---------|---------|
| Phase 1 | `cargo test -p wdrprobe-desktop -- ogexplain` | 新增集成测试全绿 |
| Phase 1 | `cargo clippy -- -D warnings` | 零警告 |
| Phase 1 | 前端 `npm run tauri:dev`，粘贴 example/opengauss_v1.html 中的 SQL | 诊断报告 JSON 返回 |
| Phase 2 | Playwright 端到端：PlanVisualizer 显示诊断图标 | 截图对比 |
| Phase 3 | `cargo test --workspace` | 全部通过（含 core + desktop） |
| Phase 4 | 前端 SQL Audit 页面：改写按钮 → diff 视图 | 改写结果正确展示 |
| Phase 5 | 旧命令标记 deprecated 后，前端无回归 | 2 周生产验证 |

---

## 七、时间线

```
Week 1-2:  Phase 1 (ogexplain-core 嵌入)
Week 2-3:  Phase 2 (前端诊断页面)  ←→  Phase 3 (core 提取) [可并行]
Week 3-4:  Phase 4 (CLI 模式)
Week 4-6:  Phase 5 (TUI 模式)
Week 6-8:  Phase 6 (metamorphosis 接入)
Week 10+:  Phase 7 (sql_parser.rs 退役，需 4 周观察期)
```

**并行机会：**
- Phase 2（前端）和 Phase 3（core 提取）互不阻塞，可并行
- Phase 4（CLI）和 Phase 6（metamorphosis）可并行（不同 crate）
- Phase 5（TUI）依赖 Phase 4 的 CLI 验证 core API 可用性，不依赖 Phase 6

**里程碑：**

| 里程碑 | 产出 | 可演示 |
|--------|------|--------|
| M1 (Week 2) | ogexplain-core 嵌入完成 | 后端诊断 JSON 返回 |
| M2 (Week 3) | 前端诊断页面 | PlanVisualizer 显示诊断图标 |
| M3 (Week 4) | core 提取 + CLI | `wdrprobe import example/opengauss_v1.html` |
| M4 (Week 6) | TUI 完成 | 终端中交互式浏览 WDR 报告 |
| M5 (Week 8) | metamorphosis 接入 | SQL 改写 diff 视图 |
| M6 (Week 12+) | sql_parser.rs 退役 | 代码库简化

---

## 八、附录：关键 API 速查

### ogexplain-core 公开 API

```rust
// 解析
pub fn parse(text: &str) -> Result<ExplainPlan, ParseError>

// 诊断
pub fn analyze(plan: &ExplainPlan) -> DiagnosticReport
pub fn analyze_with_config(plan: &ExplainPlan, config: &DiagnosticConfig) -> DiagnosticReport

// 可视化
pub fn heatmap(plan: &ExplainPlan) -> Option<PlanHeatmap>
pub fn waterfall(plan: &ExplainPlan) -> Option<PlanWaterfall>

// 改写联动
pub fn analyze_with_rewrite(plan: &ExplainPlan, sql_text: Option<&str>) -> DiagnosticReport
```

### metamorphosis-core 公开 API

```rust
pub struct RewriteEngine { registry: RuleRegistry }
impl RewriteEngine {
    pub fn new(registry: RuleRegistry) -> Self;
    pub fn rewrite(&self, ctx: &RewriteContext, stmts: Vec<Statement>) -> RewriteResult;
}

pub trait RewriteRule {
    fn id(&self) -> &'static str;
    fn safety_level(&self) -> SafetyLevel; // Safe | Conditional | Manual
    fn matches(&self, ctx: &RewriteContext, stmt: &Statement) -> MatchResult;
    fn apply(&self, ctx: &RewriteContext, stmt: &Statement) -> Option<RewriteAction>;
}
```

### 内置规则一览

| ID | 类别 | 安全级别 | 需要 Schema |
|----|------|---------|------------|
| `eliminate-select-star` | Semantic | Safe | ✅ |
| `subquery-to-join` | Performance | Conditional | ❌ |
| `detect-duplicate-eq-keys` | DataQuality | Manual | ✅ |
| `extract-candidate-values` | DataQuality | Manual | ❌ |

---

> **下一步：** 审查此规划后，按 Phase 1 开始实施。每个 Phase 结束后更新本文档的完成状态。
