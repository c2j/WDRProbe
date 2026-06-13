<div align="center">

# WDRProbe

### GaussDB / OpenGauss WDR 报告分析工具 | WDR Report Analysis Tool

A cross-platform desktop application for parsing, analyzing, and comparing GaussDB/OpenGauss WDR (Workload Diagnosis Report) files.

[Features](#-features--功能) · [Quick Start](#-quick-start--快速开始) · [Documentation](#-documentation--文档) · [Contributing](CONTRIBUTION.md)

</div>

---

## English

### Overview

WDRProbe is a **Tauri v1 desktop application** that helps DBAs and database engineers analyze GaussDB/OpenGauss WDR reports. It parses HTML-based WDR reports, stores structured data in a local SQLite database, and provides an interactive UI for performance diagnosis — including Top SQL analysis, wait event inspection, object statistics, execution plan visualization, and multi-report comparison.

### Key Features

- **WDR Report Parsing** — Supports both OpenGauss v1 and v2 HTML formats. Extracts instance metadata, efficiency metrics, load profile, Top SQL, wait events, object statistics, cache I/O stats, and database configuration.
- **WDR Report Analysis** — Drag-and-drop WDR HTML files for instant analysis without database storage. Includes risk detection, efficiency gauges, and a built-in WDR knowledge base.
- **WDR Comparison** — Compare a baseline report against one or more target reports. Analyzes SQL performance deltas, wait event changes, object stat shifts, and system metric variations with improved/degraded verdicts.
- **Execution Plan Visualizer** — Paste SQL and execution plan text to generate an interactive tree visualization. Supports GaussDB `EXPLAIN`, `EXPLAIN ANALYZE`, and tabular `EXPLAIN PERFORMANCE` formats. Includes 13 built-in analysis rules and an operator knowledge base.
- **Plan Diff** — Side-by-side comparison of two execution plans with automatic node matching, risk analysis (new/resolved risks), and overall verdict (Improved / Regressed / Similar).
- **Threshold Configuration** — Configurable alert thresholds across SQL, Wait, System, and AI categories. Includes preset templates (High Concurrency, Low Resource, Development, Production, GaussDB Optimized) and batch editing.
- **SQL Audit** — Automated detection of performance anti-patterns: full table scans, missing indexes, inefficient joins, stale statistics, expensive functions, Cartesian products, and more.
- **Audit Logging** — Full audit trail of all user operations (threshold changes, imports, exports) for compliance.
- **Bilingual UI** — Full English and Chinese (中文) interface with one-click language switching.
- **Cross-Platform** — Native builds for macOS (arm64/x86_64), Linux (x86_64/arm64), and Windows (x86_64/arm64).

### Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Framework | Tauri v1.8 |
| Backend | Rust 2021 Edition |
| Frontend | React 18 + TypeScript 5 |
| Build Tool | Vite 4 |
| Styling | TailwindCSS 3 |
| Charts | Recharts 3 |
| Database | SQLite (rusqlite + r2d2 connection pool) |
| HTML Parsing | scraper |
| SQL Parsing | nom |
| i18n | Custom React Context (EN/ZH) |

### Screenshots

> Screenshots will be added in a future release.

### Quick Start (English)

#### Prerequisites

- [Node.js](https://nodejs.org/) 18+ (LTS recommended)
- [Rust](https://www.rust-lang.org/) stable (latest)
- [Tauri CLI prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) (system dependencies for your platform)

#### Run from Source

```bash
cd Desktop
npm install
npm run tauri:dev
```

The app launches with Vite hot-reload on `http://localhost:1420`.

#### Build for Production

```bash
cd Desktop
npm run tauri:build
```

Build artifacts are placed in `Desktop/src-tauri/target/release/bundle/`.

### Download

Pre-built binaries are available on the [GitHub Releases](../../releases) page for:
- macOS: `.dmg` / `.app` (Universal + arm64 + x86_64)
- Linux: `.AppImage` / `.deb` (x86_64 + arm64)
- Windows: `.msi` / `.exe` (x86_64 + arm64)

> Linux builds target glibc 2.31 (Ubuntu 20.04) for Kylin OS compatibility.

### Documentation

| Document | Audience | Description |
|----------|----------|-------------|
| [docs/UserGuide.md](docs/UserGuide.md) | End Users | Complete feature walkthrough and usage guide |
| [docs/DeveloperGuide.md](docs/DeveloperGuide.md) | Developers | Architecture, IPC API reference, parser internals |
| [CONTRIBUTION.md](CONTRIBUTION.md) | Contributors | Dev setup, coding conventions, testing, PR process |
| [docs/desktop-IPC.md](docs/desktop-IPC.md) | Developers | Authoritative IPC interface specification (中文) |
| [docs/desktop-design.md](docs/desktop-design.md) | Designers | UI/UX design specification (中文) |

---

## 中文

### 项目简介

WDRProbe 是一款 **Tauri v1 桌面应用**，帮助 DBA 和数据库工程师分析 GaussDB/OpenGauss 的 WDR（工作负载诊断报告）。它能够解析 HTML 格式的 WDR 报告，将结构化数据存储到本地 SQLite 数据库，并通过交互式界面提供性能诊断功能——包括 Top SQL 分析、等待事件检查、对象统计、执行计划可视化和多报告对比。

### 核心功能

- **WDR 报告解析** — 同时支持 OpenGauss v1 和 v2 两种 HTML 格式。提取实例元数据、效率指标、负载概况、Top SQL、等待事件、对象统计、缓存 I/O 统计和数据库配置。
- **WDR 报告分析** — 拖拽 WDR HTML 文件即可即时分析，无需入库。包含风险检测、效率仪表盘和内置 WDR 知识库。
- **WDR 对比** — 将基准报告与一个或多个目标报告进行对比。分析 SQL 性能变化、等待事件变化、对象统计偏移和系统指标变化，给出改善/恶化评定。
- **执行计划可视化** — 粘贴 SQL 和执行计划文本，生成交互式树形可视化。支持 GaussDB `EXPLAIN`、`EXPLAIN ANALYZE` 和表格形式 `EXPLAIN PERFORMANCE`。内置 13 条分析规则和算子知识库。
- **计划比对** — 并排对比两份执行计划，自动匹配节点，进行风险分析（新增/已解决风险），给出总体评定（改善/恶化/相似）。
- **阈值配置** — 涵盖 SQL、等待、系统和 AI 四个类别的可配置告警阈值。包含预设模板（高并发、低资源、开发、生产、GaussDB 优化）和批量编辑。
- **SQL 审核** — 自动检测性能反模式：全表扫描、缺失索引、低效连接、过期统计、昂贵函数、笛卡尔积等。
- **审计日志** — 记录所有用户操作（阈值变更、导入导出等）的完整审计跟踪。
- **双语界面** — 完整的中英文界面，一键切换。
- **跨平台** — 原生构建支持 macOS（arm64/x86_64）、Linux（x86_64/arm64）和 Windows（x86_64/arm64）。

### 快速开始（中文）

#### 环境要求

- [Node.js](https://nodejs.org/) 18+（推荐 LTS）
- [Rust](https://www.rust-lang.org/) stable（最新版）
- [Tauri CLI 系统依赖](https://tauri.app/v1/guides/getting-started/prerequisites)

#### 从源码运行

```bash
cd Desktop
npm install
npm run tauri:dev
```

应用启动后，Vite 热重载服务在 `http://localhost:1420`。

#### 生产构建

```bash
cd Desktop
npm run tauri:build
```

构建产物位于 `Desktop/src-tauri/target/release/bundle/`。

### 下载

预编译的二进制文件可在 [GitHub Releases](../../releases) 页面下载：
- macOS：`.dmg` / `.app`（Universal + arm64 + x86_64）
- Linux：`.AppImage` / `.deb`（x86_64 + arm64）
- Windows：`.msi` / `.exe`（x86_64 + arm64）

> Linux 构建目标为 glibc 2.31（Ubuntu 20.04），兼容麒麟操作系统。

### 项目架构

```
WDRProbe/
├── Desktop/                    # Tauri 应用主体
│   ├── frontend/               # React + TypeScript 前端
│   │   ├── pages/              # 11 个页面组件
│   │   ├── components/         # 共享组件 (Layout, UploadDialog, ErrorBoundary...)
│   │   ├── context/            # React Context (I18n, Plan, WDR)
│   │   ├── services/           # Tauri IPC 调用封装
│   │   └── types.ts            # TypeScript 类型定义
│   ├── src-tauri/              # Rust 后端
│   │   ├── src/
│   │   │   ├── commands/       # Tauri IPC 命令 (38 个命令，7 个领域)
│   │   │   ├── database/       # SQLite 数据库层 (schema + operations)
│   │   │   ├── parsers/        # WDR HTML 解析器 + SQL 执行计划解析器
│   │   │   ├── models/         # 领域数据模型 (Serde 序列化)
│   │   │   ├── utils/          # 错误类型、审计工具、GaussDB 工具
│   │   │   └── progress/       # 进度报告
│   │   ├── tests/              # 集成测试 (21 个测试文件)
│   │   ├── Cargo.toml          # Rust 依赖
│   │   └── tauri.conf.json     # Tauri 应用配置
│   └── package.json            # 前端依赖与脚本
├── docs/                       # 设计文档 (中文)
├── specs/                      # SpecKit 工件 (规格、计划、数据模型、契约)
├── example/                    # 示例 WDR HTML 文件
├── .github/workflows/          # CI/CD 发布流水线
└── AGENTS.md                   # AI 开发助手说明
```

### 许可证

Copyright © 2025 C2J. All rights reserved.

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).

---

<div align="center">

Built with [Tauri](https://tauri.app/) · [React](https://react.dev/) · [Rust](https://www.rust-lang.org/)

</div>
