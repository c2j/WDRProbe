# WDRProbe 用户手册 | User Guide

> 版本 | Version: 0.1.0  
> 更新日期 | Updated: 2025-06

---

## 目录 | Table of Contents

1. [简介 | Introduction](#1-简介--introduction)
2. [安装与启动 | Installation](#2-安装与启动--installation)
3. [界面概览 | Interface Overview](#3-界面概览--interface-overview)
4. [WDR 报告分析 | WDR Report Analysis](#4-wdr-报告分析--wdr-report-analysis)
5. [WDR 对比 | WDR Comparison](#5-wdr-对比--wdr-comparison)
6. [执行计划可视化 | Plan Visualizer](#6-执行计划可视化--plan-visualizer)
7. [计划比对 | Plan Diff](#7-计划比对--plan-diff)
8. [阈值配置 | Threshold Configuration](#8-阈值配置--threshold-configuration)
9. [SQL 审核 | SQL Audit](#9-sql-审核--sql-audit-advanced)
10. [审计日志 | Audit Log](#10-审计日志--audit-log-advanced)
11. [常见问题 | FAQ](#11-常见问题--faq)

---

## 1. 简介 | Introduction

WDRProbe 是一款桌面应用，用于分析 GaussDB / OpenGauss 数据库的 WDR（Workload Diagnosis Report）报告。它支持直接拖拽 WDR 文件进行即时分析，无需连接数据库，所有数据在本地处理。

WDRProbe is a desktop application for analyzing GaussDB/OpenGauss WDR (Workload Diagnosis Report) files. It supports drag-and-drop WDR file analysis without database connectivity — all data is processed locally.

### 支持的报告格式 | Supported Report Formats

| 格式 | 说明 |
|------|------|
| OpenGauss v1 HTML | 第一代 HTML 格式 WDR 报告 |
| OpenGauss v2 HTML | 新版 HTML 格式 WDR 报告 |
| `.html` / `.wdr` | 支持以上扩展名 |

### 支持的语言 | Supported Languages

- 中文（默认）
- English

点击右上角语言按钮切换。Click the language button in the top-right corner to switch.

---

## 2. 安装与启动 | Installation

### 下载安装包 | Download

前往 [GitHub Releases](../releases) 下载对应平台的安装包：

| 平台 | 文件格式 |
|------|----------|
| macOS | `.dmg` 或 `.app` |
| Linux | `.AppImage` 或 `.deb` |
| Windows | `.msi` 或 `.exe` |

### macOS 安装 | macOS Installation

1. 下载 `.dmg` 文件
2. 双击打开，将 WDRProbe 拖入 Applications 文件夹
3. 首次启动时，右键点击应用 → 「打开」→ 确认信任开发者

> **注意**：如果 macOS 提示「无法验证开发者」，前往「系统设置」→「隐私与安全性」→ 点击「仍要打开」。

### Linux 安装 | Linux Installation

```bash
# AppImage 方式
chmod +x WDRProbe_*.AppImage
./WDRProbe_*.AppImage

# deb 方式
sudo dpkg -i wdrprobe_*.deb
```

### Windows 安装 | Windows Installation

双击 `.msi` 安装包，按向导完成安装。

### 数据存储位置 | Data Storage Location

应用数据存储在各平台的标准应用数据目录：

| 平台 | 路径 |
|------|------|
| macOS | `~/Library/Application Support/com.wdrprobe.desktop/` |
| Linux | `~/.local/share/com.wdrprobe.desktop/` |
| Windows | `%APPDATA%\com.wdrprobe.desktop\` |

数据库文件为 `wdrprobe.db`（SQLite）。

---

## 3. 界面概览 | Interface Overview

应用界面由以下部分组成：

```
┌─────────────────────────────────────────────┐
│  侧边栏          │  顶部导航栏               │
│                  │  WDRProbe / 当前页面      │
│  WDR 分析        ├───────────────────────────│
│  WDR 对比        │                           │
│  计划可视化      │                           │
│  计划比对        │     主内容区               │
│  阈值配置        │                           │
│                  │                           │
│  [折叠/展开]     │                           │
└─────────────────────────────────────────────┘
```

### 侧边栏菜单 | Sidebar Menu

| 菜单项 | 路由 | 功能 |
|--------|------|------|
| WDR 分析 | `/wdr-analysis` | 上传和分析 WDR 报告 |
| WDR 对比 | `/wdr-comparison` | 对比多份 WDR 报告 |
| 计划可视化 | `/visualizer` | 执行计划树形可视化 |
| 计划比对 | `/plandiff` | 两份执行计划并排比对 |
| 阈值配置 | `/thresholds` | 配置告警阈值 |

点击侧边栏底部的箭头按钮可折叠/展开侧边栏。

---

## 4. WDR 报告分析 | WDR Report Analysis

### 上传报告 | Uploading a Report

1. 进入 **WDR 分析** 页面
2. 将 WDR HTML 文件拖拽到上传区域，或点击「选择报告」按钮浏览文件
3. 应用自动解析报告内容

> **提示**：支持 `.html` 和 `.wdr` 格式。报告在客户端解析，不会自动存入数据库。

### 分析视图 | Analysis Views

报告加载后，页面显示以下标签页：

#### 概览 | Overview

- **健康评分**：基于效率指标和风险检测结果的综合评分
- **风险告警**：自动检测的问题列表（如缓存命中率低、CPU 使用率高、表膨胀、慢 SQL）
- **效率指标仪表盘**：
  - Buffer Hit %（缓存命中率）— 目标 100%
  - Effective CPU %（有效 CPU）— 目标 90%+
  - WalWrite NoWait %（WAL 无等待写）— 目标 100%
  - Soft Parse %（软解析率）— 目标 95%+
  - Non-Parse CPU %（非解析 CPU）— 越高越好
- **负载概况表**：DB Time、CPU Time、Redo Size、Logical Read 等指标的每秒/每事务/每次执行值
- **主机 CPU**：CPU 核心数、负载、用户态/系统态/IO 等待/空闲占比
- **I/O 概况**：读写请求次数和字节数
- **内存统计**：各内存组件的起始值和结束值

#### 等待事件 | Wait Events

- 可搜索、可排序的等待事件表
- 列：事件名称、等待类别、等待次数、总等待时间、平均等待时间、最大等待时间、% DB Time
- 常见等待事件说明（如 `DataFileRead` = 等待磁盘读取数据块）

#### Top SQL

- WDR 报告中耗时最高的 SQL 列表
- 支持按 **总时间 / 平均时间 / 调用次数 / CPU 时间 / IO 时间** 排序
- 支持按用户过滤和关键词搜索
- 点击某条 SQL 可查看详情面板，显示：
  - 完整 SQL 文本
  - 核心计时（总时间、CPU 时间、IO 时间、平均/最小/最大时间）
  - 行数与元组（返回行数、物理读、逻辑读）
  - 排序统计（排序次数、耗时、内存使用、溢出情况）
  - 哈希统计（哈希次数、耗时、内存使用、溢出情况）

#### 对象统计 | Object Stats

- 表和索引的访问统计
- 支持按类型（表/索引）和 Schema 过滤
- 列：Schema、对象名、类型、Seq Scan、Idx Scan、增/改/删元组、活/死元组
- 死元组数量高时表示需要 VACUUM

#### 数据库参数 | Settings

- 报告中捕获的数据库配置参数
- 可搜索、可排序

### WDR 知识库 | WDR Knowledge Base

页面右侧/底部的知识库面板提供了 WDR 指标的详细解释，帮助理解各项指标的含义和建议值。

---

## 5. WDR 对比 | WDR Comparison

### 创建对比 | Creating a Comparison

1. 进入 **WDR 对比** 页面
2. 在 **基准报告** 区域上传一份 WDR HTML 文件
3. 在 **对比报告** 区域上传一份或多份目标 WDR 文件
4. 点击「添加对比报告」可添加更多目标报告

### 对比视图 | Comparison Views

#### 关键指标 | Key Metrics

- 负载概况和效率指标的并排对比
- 变化率以颜色标识：绿色 = 改善，红色 = 恶化
- 支持一键重置所有对比

#### Top 等待事件 | Top Wait Events

- 等待事件对比表，显示基准值和各目标的等待次数、平均时间、最大时间
- 支持排序方式：按总耗时 / 按平均耗时 / 按差异幅度 / 按频率变化
- 检测到的锁等待会以告警标识

#### Top 20 SQL | Top SQL

- SQL 级别的性能对比
- 支持排序方式：按总耗时 / 按平均耗时 / 按差异幅度 / 按执行频率
- 显示 Unique SQL ID、执行次数、总/平均/CPU/IO 时间、行数、逻辑读
- 点击某条 SQL 查看详细对比面板
- 支持按用户过滤

### 对比结果解读 | Interpreting Results

| 标识 | 含义 |
|------|------|
| 绿色数值 ↓ | 性能改善（如耗时下降） |
| 红色数值 ↑ | 性能恶化（如耗时上升） |
| 灰色 | 变化不显著 |
| 「未找到」 | 该 SQL 仅在一份报告中出现 |

---

## 6. 执行计划可视化 | Plan Visualizer

### 使用步骤 | How to Use

1. 进入 **计划可视化** 页面
2. 在 **SQL 编辑器** 中粘贴你的 SQL 语句（可选）
3. 在 **计划文本** 框中粘贴执行计划
   - 支持 GaussDB `EXPLAIN` 输出
   - 支持 `EXPLAIN ANALYZE` 输出（含实际执行统计）
   - 支持 `EXPLAIN PERFORMANCE` 表格格式
4. 点击「Explain」按钮生成可视化

### 可视化模式 | View Modes

#### 树形视图 | Tree View

- 可折叠/展开的节点树
- 每个节点显示：算子名称、代价、行数
- 如果是 `EXPLAIN ANALYZE`，显示实际时间和行数
- **高代价节点高亮**：占总代价 >20% 的节点标红
- **下盘标识**：检测到 Disk Spill 的节点显示警告图标
- **CTE 高亮**：悬停在 CTE Scan 上时高亮对应的 CTE 定义
- 支持缩放、适应屏幕、重置视图

#### 代价流视图 | Cost Flow View

- 以瀑布图方式展示各节点的代价占比

### 分析规则 | Analysis Rules

可视化器内置 13 条分析规则，自动检测以下问题：

| 规则 | 说明 |
|------|------|
| 总代价过高 | 计划总代价超过阈值 |
| 大表全表扫描 | Seq Scan 扫描行数过多 |
| SubPlan | 检测到子计划，建议改写为 JOIN |
| 笛卡尔积 | Nested Loop 无索引，可能产生笛卡尔积 |
| 分区扫描过多 | 扫描了大量分区，检查分区剪枝 |
| Bitmap Scan | 使用了 Bitmap Scan，检查是否 Index Scan 更优 |
| 下盘 | 操作溢出到磁盘，建议增加 `work_mem` |
| 索引扫描带过滤 | 索引扫描包含过滤条件，建议将过滤列加入索引 |
| 用户函数 | 计划中包含用户函数，检查函数性能 |
| 复杂更新 | UPDATE 包含多个子查询 |
| Rownum 限制 | 大结果集上使用 ROWNUM |
| 执行时间长 | 实际执行时间过长 |
| 代价极高 | 单节点代价极高 |

### 算子知识库 | Operator Knowledge Base

知识库面板提供了常见算子的详细说明，包括：

- **扫描算子**：Seq Scan、Index Scan、Index Only Scan、Bitmap Scan、CTE Scan、Subquery Scan
- **连接算子**：Hash Join、Nested Loop、Merge Join
- **其他算子**：Sort、Aggregate、Limit、Append、Materialize、Partition Iterator、Result
- **Hint 知识库**：leading、join 方法、rows、scan 方法、stream、blockname 等 GaussDB Hint 的使用说明

在搜索框中输入算子名称即可快速查找。

---

## 7. 计划比对 | Plan Diff

### 使用步骤 | How to Use

1. 进入 **计划比对** 页面
2. 选择输入模式：
   - **统一输入**：将两份计划粘贴在同一文本框中，工具自动拆分
   - **分栏输入**：分别粘贴基准计划和目标计划
3. 点击「比对」按钮

### 比对结果 | Diff Results

#### KPI 卡片

显示基准和目标的对比：
- 总代价 | Total Cost
- 执行时间 | Execution Time
- 行数 | Rows
- 每项指标的变化百分比

#### 评定标识 | Verdict Badge

| 评定 | 含义 |
|------|------|
| 改善 (Improved) | 目标计划性能优于基准 |
| 恶化 (Regressed) | 目标计划性能劣于基准 |
| 相似 (Similar) | 两份计划性能接近 |
| 新增 (New) | 节点仅出现在目标计划中 |
| 移除 (Removed) | 节点仅出现在基准计划中 |

#### 风险分析 | Risk Analysis

自动检测并对比以下风险：
- 下盘检测
- 大表全表扫描（标注行数）
- 代价极高
- 大数据量嵌套循环
- 行数估算严重偏差

对于每个风险，标注其状态：**新增风险**、**已解决**、或 **未变化**。

### 节点匹配 | Node Matching

工具使用启发式算法自动匹配两份计划中的对应节点（基于算子名称、目标对象、代价、节点 ID）。匹配的节点之间会用连线标识，点击可查看详细分析卡片。

---

## 8. 阈值配置 | Threshold Configuration

### 分类 | Categories

| 分类 | 说明 | 示例阈值 |
|------|------|----------|
| SQL | SQL 性能相关 | 慢 SQL 时间、全表扫描行数、CPU 时间、IO 时间、Buffer Gets |
| WAIT | 等待事件相关 | 最大锁等待、最大 IO 等待、最大 LWLock 等待 |
| SYSTEM | 系统资源相关 | CPU 使用率、内存使用率、磁盘 IO、缓存命中率 |
| AI | AI 分析相关 | 采样大小、置信度 |

### 编辑阈值 | Editing Thresholds

1. 在左侧选择分类
2. 在右侧表格中直接编辑「值」列
3. 点击「批量保存」保存所有修改

每个阈值都标注了推荐范围（最小值 ~ 最大值），修改时会自动验证。

### 模板管理 | Template Management

系统提供以下预设模板，可一键应用：

| 模板 | 适用场景 |
|------|----------|
| 高并发模板 | 高并发在线交易系统 |
| 低资源模板 | 资源受限环境 |
| 开发模板 | 开发测试环境 |
| 生产模板 | 生产环境推荐值 |
| GaussDB 优化模板 | GaussDB 特定优化 |

---

## 9. SQL 审核 | SQL Audit (Advanced)

> 此功能通过 URL `#/sqlaudit` 访问（当前未在侧边栏显示）。

### 功能说明

- 查看自动检测到的 SQL 性能问题
- 按状态过滤：全部、待处理、处理中、已修复、已白名单
- 每个问题显示：严重程度（高/中/低）、类型、目标 SQL、发现时间、状态

### 优化建议 | Optimization Recommendations

点击「优化」按钮打开优化建议弹窗：
- **诊断**：解释问题原因
- **建议**：提供具体的优化方案（如创建索引、改写 SQL）
- 可选择「加入白名单」（忽略此问题）或「应用」（采纳建议）

### 检测规则 | Detection Rules

| 问题类型 | 严重程度 | 说明 |
|----------|----------|------|
| FullTableScan | 严重 | 大表全表扫描 |
| MissingIndex | 高 | 缺失连接索引 |
| InefficientJoin | 中 | 低效嵌套循环连接 |
| ExpensiveFunction | 中 | 昂贵的函数调用 |
| MissingStats | 高 | 统计信息过期或缺失 |
| CartesianProduct | 高 | 笛卡尔积 |
| SortOperation | 中 | 大量排序操作 |

---

## 10. 审计日志 | Audit Log (Advanced)

> 此功能通过 URL `#/auditlog` 访问（当前未在侧边栏显示）。

### 功能说明

- 查看所有用户操作的审计记录
- 按操作类型过滤（全部操作、更新阈值、导出等）
- 按日期过滤
- 支持导出

### 日志内容 | Log Contents

| 列 | 说明 |
|----|------|
| 时间 | 操作发生时间 |
| 用户 | 执行操作的用户 |
| 操作类型 | 如「更新阈值」「应用模板」「导出」|
| 对象 | 操作目标 |
| 结果 | 成功 / 失败 |

---

## 11. 常见问题 | FAQ

### Q: 支持哪些数据库的 WDR 报告？

A: 目前支持 GaussDB 和 OpenGauss 的 WDR HTML 报告，包括 v1 和 v2 两种格式。

### Q: WDR 文件在哪里获取？

A: 在 GaussDB/OpenGauss 中执行以下命令生成 WDR 报告：
```sql
-- 在数据库中执行
SELECT * FROM pgxc_generate_wdr_report(
  begin_snap_id,    -- 起始快照 ID
  end_snap_id,      -- 结束快照 ID
  'html',           -- 输出格式
  'all',            -- 报告类型
  '/path/to/report.html'  -- 输出路径
);
```

### Q: 上传的 WDR 文件数据是否会上传到服务器？

A: 不会。WDRProbe 是纯本地应用，所有数据解析和存储都在本地完成，不会传输到任何服务器。

### Q: 对比时，基准报告和目标报告有什么区别？

A: 基准报告是参照物（通常是优化前的报告），目标报告是对比对象（通常是优化后的报告）。工具计算目标相对于基准的变化量。

### Q: 执行计划可视化支持哪些格式？

A: 支持 GaussDB 的三种 EXPLAIN 输出：
- `EXPLAIN`（仅计划）
- `EXPLAIN ANALYZE`（含实际执行统计）
- `EXPLAIN PERFORMANCE`（表格格式，GaussDB 特有）

### Q: 如何切换界面语言？

A: 点击右上角的语言按钮（显示「EN」或「中文」）即可一键切换中英文。

### Q: 应用数据存在哪里？如何清理？

A: 数据存储在各平台的标准应用数据目录下（见 [安装与启动](#2-安装与启动--installation)）。删除 `wdrprobe.db` 文件即可清空所有数据。

---

### Q: Which database WDR reports are supported?

A: Currently GaussDB and OpenGauss WDR HTML reports are supported, including both v1 and v2 formats.

### Q: Where do I get WDR files?

A: Generate WDR reports in GaussDB/OpenGauss:
```sql
SELECT * FROM pgxc_generate_wdr_report(
  begin_snap_id, end_snap_id, 'html', 'all', '/path/to/report.html'
);
```

### Q: Does WDRProbe upload data to any server?

A: No. WDRProbe is fully local — all parsing and storage happens on your machine.

### Q: How do I switch the interface language?

A: Click the language button in the top-right corner (shows "EN" or "中文").

---

> 如需更多帮助，请提交 [GitHub Issue](../issues)。  
> For more help, please submit a [GitHub Issue](../issues).
