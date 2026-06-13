use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyEvent};
use wdrprobe_core::DatabaseOperations;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Clear, Paragraph};
use ratatui::Frame;

use crate::components::tree::FlatNode;
use crate::theme::Theme;
use crate::ui::plan_view;

/// Represents the 5 TUI pages
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Reports,
    ReportDetail,
    PlanView,
    Audit,
}

impl Page {
    pub fn next(self) -> Self {
        match self {
            Page::Dashboard => Page::Reports,
            Page::Reports => Page::ReportDetail,
            Page::ReportDetail => Page::PlanView,
            Page::PlanView => Page::Audit,
            Page::Audit => Page::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Page::Dashboard => Page::Audit,
            Page::Reports => Page::Dashboard,
            Page::ReportDetail => Page::Reports,
            Page::PlanView => Page::ReportDetail,
            Page::Audit => Page::PlanView,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Reports => "Reports",
            Page::ReportDetail => "Report Detail",
            Page::PlanView => "Plan View",
            Page::Audit => "Audit",
        }
    }
}

/// A row for audit issues, with necessary display fields
#[derive(Debug, Clone)]
pub struct AuditIssueRow {
    pub id: i64,
    pub report_id: Option<i64>,
    pub sql_id: Option<i64>,
    pub issue_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub problematic_sql: Option<String>,
    pub recommendation: String,
    pub status: String,
    pub detected_at: String,
}

/// Main application state
pub struct App {
    pub should_quit: bool,
    pub show_help: bool,
    pub current_page: Page,
    pub db_path: String,
    pub pool: Option<wdrprobe_core::database::DatabasePool>,

    // Reports list state
    pub reports: Vec<wdrprobe_core::models::report::WdrReport>,
    pub reports_state: ratatui::widgets::ListState,
    pub selected_report_id: Option<i64>,

    // Report detail data
    pub detail_report: Option<wdrprobe_core::models::report::WdrReport>,
    pub detail_efficiency: Option<wdrprobe_core::models::report::EfficiencyMetrics>,
    pub detail_load_profile: Option<wdrprobe_core::models::report::LoadProfile>,
    pub detail_top_sqls: Vec<wdrprobe_core::models::report::TopSql>,

    // Plan view state
    pub plan_nodes: Vec<FlatNode>,
    pub plan_expanded: HashSet<usize>,
    pub plan_selected: usize,

    // Audit data
    pub audit_issues: Vec<AuditIssueRow>,
    pub audit_selected: usize,

    // Scroll for detail
    pub detail_scroll: usize,
}

impl App {
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        Ok(App {
            should_quit: false,
            show_help: false,
            current_page: Page::Dashboard,
            db_path: db_path.to_string(),
            pool: None,
            reports: Vec::new(),
            reports_state: ratatui::widgets::ListState::default(),
            selected_report_id: None,
            detail_report: None,
            detail_efficiency: None,
            detail_load_profile: None,
            detail_top_sqls: Vec::new(),
            plan_nodes: Vec::new(),
            plan_expanded: HashSet::new(),
            plan_selected: 0,
            audit_issues: Vec::new(),
            audit_selected: 0,
            detail_scroll: 0,
        })
    }

    /// Load reports and initial data from the DB
    pub fn load_data(&mut self) -> anyhow::Result<()> {
        let pool = wdrprobe_core::database::init_database(&self.db_path)
            .map_err(|e| anyhow::anyhow!("DB init failed: {}", e))?;
        let conn = wdrprobe_core::database::get_connection(&pool)
            .map_err(|e| anyhow::anyhow!("DB connection failed: {}", e))?;
        wdrprobe_core::database::initialize_schema(&conn)
            .map_err(|e| anyhow::anyhow!("Schema init failed: {}", e))?;

        self.reports = pool
            .list_wdr_reports(None, None)
            .map_err(|e| anyhow::anyhow!("Failed to list reports: {}", e))?;

        if !self.reports.is_empty() {
            self.reports_state.select(Some(0));
        }

        // Re-insert the connection before storing pool
        drop(conn);
        self.pool = Some(pool);
        Ok(())
    }

    /// Select a report and load its detail data
    pub fn select_report(&mut self) {
        if let Some(idx) = self.reports_state.selected() {
            if idx < self.reports.len() {
                let report_id = self.reports[idx].id;
                self.selected_report_id = Some(report_id);

                if let Some(ref pool) = self.pool {
                    self.detail_report = pool.get_wdr_report(report_id).ok().flatten();
                    self.detail_efficiency = pool.get_efficiency_metrics(report_id).ok().flatten();
                    self.detail_load_profile = pool.get_load_profile(report_id).ok().flatten();
                    self.detail_top_sqls =
                        pool.get_top_sqls_by_report(report_id).unwrap_or_default();

                    // Load audit issues for this report
                    self.load_audit_issues(report_id);

                    // Load execution plans and build tree
                    self.load_plan_tree(report_id);
                }

                self.current_page = Page::ReportDetail;
                self.detail_scroll = 0;
            }
        }
    }

    /// Load audit issues from the DB for a given report
    fn load_audit_issues(&mut self, report_id: i64) {
        self.audit_issues.clear();
        self.audit_selected = 0;

        if let Some(ref pool) = self.pool {
            if let Ok(conn) = wdrprobe_core::database::get_connection(pool) {
                let query = "
                    SELECT id, report_id, sql_id, issue_type, severity, title, description,
                           problematic_sql, recommendation, status, detected_at
                    FROM sql_audit_issues
                    WHERE report_id = ?
                    ORDER BY
                      CASE severity
                        WHEN 'Critical' THEN 0 WHEN 'High' THEN 1
                        WHEN 'Medium' THEN 2 WHEN 'Low' THEN 3 ELSE 4
                      END,
                      detected_at DESC
                ";
                if let Ok(mut stmt) = conn.prepare(query) {
                    if let Ok(rows) = stmt.query_map(rusqlite::params![report_id], |row| {
                        Ok(AuditIssueRow {
                            id: row.get("id")?,
                            report_id: row.get("report_id")?,
                            sql_id: row.get("sql_id")?,
                            issue_type: row.get("issue_type")?,
                            severity: row.get("severity")?,
                            title: row.get("title")?,
                            description: row.get("description")?,
                            problematic_sql: row.get("problematic_sql")?,
                            recommendation: row.get("recommendation")?,
                            status: row.get("status")?,
                            detected_at: row.get("detected_at")?,
                        })
                    }) {
                        for row in rows.flatten() {
                            self.audit_issues.push(row);
                        }
                    }
                }
            }
        }
    }

    /// Load execution plans and build the flat tree
    fn load_plan_tree(&mut self, report_id: i64) {
        if let Some(ref pool) = self.pool {
            let plans = pool
                .get_execution_plans_by_report(report_id)
                .unwrap_or_default();
            self.plan_nodes = plan_view::build_plan_nodes(&plans);
            // Initialize expanded set from the flat nodes
            self.plan_expanded.clear();
            for node in &self.plan_nodes {
                if node.has_children && node.depth < 1 {
                    self.plan_expanded.insert(node.id);
                }
            }
            self.plan_selected = 0;
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) {
        // Help overlay takes priority
        if self.show_help {
            if key.code == KeyCode::Char('?') {
                self.show_help = false;
            }
            return;
        }

        // Global keys
        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return;
            }
            KeyCode::Char('?') => {
                self.show_help = true;
                return;
            }
            KeyCode::Tab => {
                self.current_page = self.current_page.next();
                return;
            }
            KeyCode::BackTab => {
                self.current_page = self.current_page.prev();
                return;
            }
            _ => {}
        }

        // Page-specific keys
        match self.current_page {
            Page::Dashboard => {
                // No interactive navigation on dashboard
            }
            Page::Reports => self.handle_reports_key(key),
            Page::ReportDetail => self.handle_detail_key(key),
            Page::PlanView => self.handle_plan_key(key),
            Page::Audit => self.handle_audit_key(key),
        }
    }

    fn handle_reports_key(&mut self, key: KeyEvent) {
        let len = self.reports.len();
        if len == 0 {
            return;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                let sel = self.reports_state.selected().unwrap_or(0);
                self.reports_state.select(Some((sel + 1).min(len - 1)));
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let sel = self.reports_state.selected().unwrap_or(0);
                self.reports_state.select(Some(sel.saturating_sub(1)));
            }
            KeyCode::Enter => {
                self.select_report();
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.detail_scroll = self.detail_scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.detail_scroll = self.detail_scroll.saturating_sub(1);
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_page = Page::Reports;
                self.detail_scroll = 0;
            }
            _ => {}
        }
    }

    fn handle_plan_key(&mut self, key: KeyEvent) {
        let len = self.plan_nodes.len();
        if len == 0 {
            return;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.plan_selected = (self.plan_selected + 1).min(len - 1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.plan_selected = self.plan_selected.saturating_sub(1);
            }
            KeyCode::Enter => {
                plan_view::toggle_node(&self.plan_nodes, self.plan_selected, &mut self.plan_expanded);
            }
            KeyCode::Char('e') | KeyCode::Char('E') => {
                plan_view::expand_all(&self.plan_nodes, &mut self.plan_expanded);
            }
            KeyCode::Char('w') | KeyCode::Char('W') => {
                plan_view::collapse_all(&self.plan_nodes, &mut self.plan_expanded);
            }
            KeyCode::Esc | KeyCode::Backspace => {
                self.current_page = Page::ReportDetail;
            }
            _ => {}
        }
    }

    fn handle_audit_key(&mut self, key: KeyEvent) {
        let len = self.audit_issues.len();
        if len == 0 {
            return;
        }
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => {
                self.audit_selected = (self.audit_selected + 1).min(len - 1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.audit_selected = self.audit_selected.saturating_sub(1);
            }
            _ => {}
        }
    }

    /// Draw the entire TUI
    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Layout: title (1) + main (flex) + status (1)
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        // Title bar
        let title = format!(
            " WDRProbe TUI  |  Page: {}  |  DB: {} ",
            self.current_page.title(),
            self.db_path
        );
        frame.render_widget(
            Paragraph::new(title).style(Theme::title_bar()),
            chunks[0],
        );

        // Main content
        match self.current_page {
            Page::Dashboard => crate::ui::dashboard::render(frame, chunks[1], self),
            Page::Reports => crate::ui::reports::render(frame, chunks[1], self),
            Page::ReportDetail => crate::ui::report_detail::render(frame, chunks[1], self),
            Page::PlanView => crate::ui::plan_view::render(frame, chunks[1], self),
            Page::Audit => crate::ui::audit::render(frame, chunks[1], self),
        }

        // Status bar
        let status =
            " Tab:Switch  j/k:Navigate  Enter:Select  Esc:Back  ?:Help  q:Quit ";
        frame.render_widget(
            Paragraph::new(status).style(Theme::status_bar()),
            chunks[2],
        );

        // Help overlay
        if self.show_help {
            self.render_help(frame, area);
        }
    }

    /// Render a centered help popup
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            "=== WDRProbe TUI Keybindings ===",
            "",
            "Global:",
            "  Tab / Shift+Tab  Switch page",
            "  q                Quit",
            "  ?                Toggle this help",
            "",
            "Reports:",
            "  j/k or up/down  Navigate reports",
            "  Enter            View report details",
            "",
            "Report Detail:",
            "  j/k              Scroll content",
            "  Esc/Backspace    Back to reports list",
            "",
            "Plan View:",
            "  j/k              Navigate tree nodes",
            "  Enter            Expand/collapse node",
            "  E                Expand all nodes",
            "  W                Collapse all nodes",
            "",
            "Audit:",
            "  j/k              Navigate audit issues",
        ];

        let popup_area = centered_rect(60, 70, area);

        // Clear background
        frame.render_widget(Clear, popup_area);

        // Render help block
        let block = Block::bordered()
            .title(" Help (press ? to close) ")
            .style(Style::default().fg(Color::Cyan));

        let paragraph = Paragraph::new(help_text.join("\n")).block(block);
        frame.render_widget(paragraph, popup_area);
    }
}

/// Create a centered rectangle (percentage-based)
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

/// Run the TUI application
pub fn run_app(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    db_path: &str,
) -> anyhow::Result<()> {
    let mut app = App::new(db_path)?;

    // Load initial data (best-effort: continue even if DB is empty/new)
    if let Err(e) = app.load_data() {
        // Just log the error, the TUI will show "no data" pages
        eprintln!("Warning: Failed to load data: {}", e);
    }

    loop {
        terminal.draw(|f| app.draw(f))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                app.handle_key(key);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
