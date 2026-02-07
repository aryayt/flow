use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use flow_core::{Feature, Theme};
use flow_db::Database;
use ratatui::Frame;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::theme::TuiTheme;
use crate::views;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Kanban,
    Agents,
    Logs,
    Graph,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Full,
    Compact,
    Mobile,
}

pub struct App {
    pub current_view: View,
    pub layout_mode: LayoutMode,
    pub theme: Theme,
    pub tui_theme: TuiTheme,
    pub features: Vec<Feature>,
    pub selected_index: usize,
    pub show_help: bool,
    #[allow(dead_code)]
    pub should_quit: bool,
    pub terminal_width: u16,
    pub terminal_height: u16,
    #[allow(dead_code)]
    pub scroll_offset: usize,
    pub log_messages: Vec<String>,
    pub db: Option<Arc<Database>>,
    pub db_path: Option<PathBuf>,
    pub db_error: Option<String>,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
    pub status_message: Option<(String, Instant)>,
    pub feature_stats: Option<flow_core::FeatureStats>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_db_path(None)
    }

    pub fn with_db_path(db_path: Option<PathBuf>) -> Self {
        let theme = Theme::default();
        let tui_theme = TuiTheme::from(&theme.colors());

        let mut app = Self {
            current_view: View::Kanban,
            layout_mode: LayoutMode::Full,
            theme,
            tui_theme,
            features: Vec::new(),
            selected_index: 0,
            show_help: false,
            should_quit: false,
            terminal_width: 120,
            terminal_height: 30,
            scroll_offset: 0,
            log_messages: Vec::new(),
            db: None,
            db_path,
            db_error: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            status_message: None,
            feature_stats: None,
        };

        app.load_from_database();
        app
    }

    pub fn render(&self, frame: &mut Frame) {
        if self.show_help {
            views::help::render(frame, self);
        } else {
            match self.current_view {
                View::Kanban => views::kanban::render(frame, self),
                View::Agents => views::agents::render(frame, self),
                View::Logs => views::logs::render(frame, self),
                View::Graph => views::graph::render(frame, self),
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        // Quit commands
        if key.code == KeyCode::Char('q')
            || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            return true;
        }

        // Help toggle
        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return false;
        }

        // If help is showing, only toggle it off
        if self.show_help {
            self.show_help = false;
            return false;
        }

        // View switching
        match key.code {
            KeyCode::Char('1') => self.current_view = View::Kanban,
            KeyCode::Char('2') => self.current_view = View::Agents,
            KeyCode::Char('3') => self.current_view = View::Logs,
            KeyCode::Char('4') => self.current_view = View::Graph,
            KeyCode::Tab => self.next_view(),
            _ => {}
        }

        // Navigation
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.next_item(),
            KeyCode::Char('k') | KeyCode::Up => self.prev_item(),
            _ => {}
        }

        // Theme cycling
        if key.code == KeyCode::Char('t') {
            self.cycle_theme();
        }

        // Database actions
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => self.claim_selected_feature(),
            KeyCode::Char('p') => self.mark_selected_passing(),
            KeyCode::Char('f') => self.mark_selected_failing(),
            KeyCode::Char('c') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.clear_selected_in_progress();
            }
            KeyCode::Char('r') => self.manual_refresh(),
            _ => {}
        }

        false
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.terminal_width = width;
        self.terminal_height = height;

        // Update layout mode based on width
        self.layout_mode = if width >= 120 {
            LayoutMode::Full
        } else if width >= 80 {
            LayoutMode::Compact
        } else {
            LayoutMode::Mobile
        };
    }

    fn next_view(&mut self) {
        self.current_view = match self.current_view {
            View::Kanban => View::Agents,
            View::Agents => View::Logs,
            View::Logs => View::Graph,
            View::Graph => View::Kanban,
        };
    }

    fn next_item(&mut self) {
        if !self.features.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.features.len();
        }
    }

    fn prev_item(&mut self) {
        if !self.features.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.features.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.tui_theme = TuiTheme::from(&self.theme.colors());
        self.add_log(format!("Theme changed to: {}", self.theme.name()));
    }

    pub fn add_log(&mut self, message: String) {
        self.log_messages.push(message);
        // Keep last 100 messages
        if self.log_messages.len() > 100 {
            self.log_messages.remove(0);
        }
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }

    pub fn get_status_message(&self) -> Option<&str> {
        if let Some((msg, timestamp)) = &self.status_message {
            // Show status for 3 seconds
            if timestamp.elapsed() < Duration::from_secs(3) {
                return Some(msg.as_str());
            }
        }
        None
    }

    pub fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.refresh_interval
    }

    pub fn auto_refresh(&mut self) {
        if self.should_refresh() {
            self.load_from_database();
        }
    }

    pub fn load_from_database(&mut self) {
        self.last_refresh = Instant::now();

        // Determine database path
        let db_path = if let Some(ref path) = self.db_path {
            path.clone()
        } else {
            // Try default path: ~/.agent-flow/flow.db
            let Ok(home) = std::env::var("HOME") else {
                self.db_error = Some("HOME environment variable not set".to_string());
                self.load_demo_data();
                return;
            };
            PathBuf::from(home).join(".agent-flow").join("flow.db")
        };

        // Try to open database
        match Database::open(&db_path) {
            Ok(db) => {
                let db = Arc::new(db);
                self.db = Some(Arc::clone(&db));
                self.db_path = Some(db_path.clone());
                self.db_error = None;

                // Load features
                if let Ok(conn) = db.writer().lock() {
                    match flow_db::FeatureStore::get_all(&conn) {
                        Ok(features) => {
                            self.features = features;
                            if self.selected_index >= self.features.len()
                                && !self.features.is_empty()
                            {
                                self.selected_index = self.features.len() - 1;
                            }
                        }
                        Err(e) => {
                            self.db_error = Some(format!("Failed to load features: {e}"));
                            self.add_log(format!("Error loading features: {e}"));
                        }
                    }

                    // Load stats
                    match flow_db::FeatureStore::get_stats(&conn) {
                        Ok(stats) => {
                            self.feature_stats = Some(stats);
                        }
                        Err(e) => {
                            self.add_log(format!("Error loading stats: {e}"));
                        }
                    }
                } else {
                    self.db_error = Some("Failed to acquire database lock".to_string());
                }

                self.add_log(format!(
                    "Loaded {} features from database",
                    self.features.len()
                ));
            }
            Err(e) => {
                self.db_error = Some(format!("Failed to open database: {e}"));
                self.add_log("No database found. Run 'flow init' to create one, or 'flow features add' to add features.".to_string());
                self.load_demo_data();
            }
        }
    }

    fn manual_refresh(&mut self) {
        self.load_from_database();
        self.set_status("✓ Refreshed from database".to_string());
    }

    fn claim_selected_feature(&mut self) {
        if self.features.is_empty() {
            return;
        }

        let feature_id = self.features[self.selected_index].id;
        let feature_name = self.features[self.selected_index].name.clone();

        let db = match &self.db {
            Some(db) => Arc::clone(db),
            None => return,
        };

        let result = {
            if let Ok(conn) = db.writer().lock() {
                flow_db::FeatureStore::claim_and_get(&conn, feature_id)
            } else {
                return;
            }
        };

        match result {
            Ok(_) => {
                self.set_status(format!("✓ Claimed \"{feature_name}\" (#{feature_id})"));
                self.add_log(format!("Claimed feature #{feature_id}: {feature_name}"));
                self.load_from_database();
            }
            Err(e) => {
                self.set_status(format!("✗ Failed to claim: {e}"));
                self.add_log(format!("Failed to claim feature #{feature_id}: {e}"));
            }
        }
    }

    fn mark_selected_passing(&mut self) {
        if self.features.is_empty() {
            return;
        }

        let feature_id = self.features[self.selected_index].id;
        let feature_name = self.features[self.selected_index].name.clone();

        let db = match &self.db {
            Some(db) => Arc::clone(db),
            None => return,
        };

        let result = {
            if let Ok(conn) = db.writer().lock() {
                flow_db::FeatureStore::mark_passing(&conn, feature_id)
            } else {
                return;
            }
        };

        match result {
            Ok(()) => {
                self.set_status(format!("✓ Marked \"{feature_name}\" as passing"));
                self.add_log(format!("Marked feature #{feature_id} as passing"));
                self.load_from_database();
            }
            Err(e) => {
                self.set_status(format!("✗ Failed to mark passing: {e}"));
                self.add_log(format!(
                    "Failed to mark feature #{feature_id} as passing: {e}"
                ));
            }
        }
    }

    fn mark_selected_failing(&mut self) {
        if self.features.is_empty() {
            return;
        }

        let feature_id = self.features[self.selected_index].id;
        let feature_name = self.features[self.selected_index].name.clone();

        let db = match &self.db {
            Some(db) => Arc::clone(db),
            None => return,
        };

        let result = {
            if let Ok(conn) = db.writer().lock() {
                flow_db::FeatureStore::mark_failing(&conn, feature_id)
            } else {
                return;
            }
        };

        match result {
            Ok(()) => {
                self.set_status(format!("✓ Marked \"{feature_name}\" as failing"));
                self.add_log(format!("Marked feature #{feature_id} as failing"));
                self.load_from_database();
            }
            Err(e) => {
                self.set_status(format!("✗ Failed to mark failing: {e}"));
                self.add_log(format!(
                    "Failed to mark feature #{feature_id} as failing: {e}"
                ));
            }
        }
    }

    fn clear_selected_in_progress(&mut self) {
        if self.features.is_empty() {
            return;
        }

        let feature_id = self.features[self.selected_index].id;
        let feature_name = self.features[self.selected_index].name.clone();

        let db = match &self.db {
            Some(db) => Arc::clone(db),
            None => return,
        };

        let result = {
            if let Ok(conn) = db.writer().lock() {
                flow_db::FeatureStore::clear_in_progress(&conn, feature_id)
            } else {
                return;
            }
        };

        match result {
            Ok(()) => {
                self.set_status(format!("✓ Cleared in-progress for \"{feature_name}\""));
                self.add_log(format!("Cleared in-progress for feature #{feature_id}"));
                self.load_from_database();
            }
            Err(e) => {
                self.set_status(format!("✗ Failed to clear in-progress: {e}"));
                self.add_log(format!(
                    "Failed to clear in-progress for feature #{feature_id}: {e}"
                ));
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn load_demo_data(&mut self) {
        // Create demo features for testing
        self.features = vec![
            Feature {
                id: 1,
                priority: 100,
                category: "Backend".to_string(),
                name: "User Authentication".to_string(),
                description: "Implement JWT-based authentication system".to_string(),
                steps: vec![
                    "Create user model".to_string(),
                    "Implement JWT tokens".to_string(),
                    "Add login/logout endpoints".to_string(),
                ],
                passes: false,
                in_progress: false,
                dependencies: vec![],
                created_at: Some("2024-01-15T10:00:00Z".to_string()),
                updated_at: Some("2024-01-15T10:00:00Z".to_string()),
            },
            Feature {
                id: 2,
                priority: 90,
                category: "Frontend".to_string(),
                name: "Login Page UI".to_string(),
                description: "Create responsive login page".to_string(),
                steps: vec![
                    "Design mockup".to_string(),
                    "Implement form".to_string(),
                    "Add validation".to_string(),
                ],
                passes: false,
                in_progress: false,
                dependencies: vec![],
                created_at: Some("2024-01-15T11:00:00Z".to_string()),
                updated_at: Some("2024-01-15T11:00:00Z".to_string()),
            },
            Feature {
                id: 3,
                priority: 80,
                category: "Backend".to_string(),
                name: "API Rate Limiting".to_string(),
                description: "Add rate limiting middleware".to_string(),
                steps: vec![
                    "Research solutions".to_string(),
                    "Implement middleware".to_string(),
                ],
                passes: false,
                in_progress: true,
                dependencies: vec![1],
                created_at: Some("2024-01-16T09:00:00Z".to_string()),
                updated_at: Some("2024-01-16T14:30:00Z".to_string()),
            },
            Feature {
                id: 4,
                priority: 70,
                category: "Testing".to_string(),
                name: "Integration Tests".to_string(),
                description: "Write comprehensive test suite".to_string(),
                steps: vec![
                    "Setup test framework".to_string(),
                    "Write API tests".to_string(),
                    "Add CI/CD integration".to_string(),
                ],
                passes: false,
                in_progress: true,
                dependencies: vec![1, 3],
                created_at: Some("2024-01-16T10:00:00Z".to_string()),
                updated_at: Some("2024-01-16T15:00:00Z".to_string()),
            },
            Feature {
                id: 5,
                priority: 60,
                category: "Frontend".to_string(),
                name: "Dashboard".to_string(),
                description: "User dashboard with stats".to_string(),
                steps: vec![
                    "Create layout".to_string(),
                    "Add charts".to_string(),
                    "Implement data fetching".to_string(),
                ],
                passes: true,
                in_progress: false,
                dependencies: vec![1, 2],
                created_at: Some("2024-01-14T08:00:00Z".to_string()),
                updated_at: Some("2024-01-15T17:00:00Z".to_string()),
            },
            Feature {
                id: 6,
                priority: 50,
                category: "Backend".to_string(),
                name: "Email Notifications".to_string(),
                description: "Send email notifications for events".to_string(),
                steps: vec![
                    "Setup email service".to_string(),
                    "Create templates".to_string(),
                ],
                passes: true,
                in_progress: false,
                dependencies: vec![1],
                created_at: Some("2024-01-13T14:00:00Z".to_string()),
                updated_at: Some("2024-01-14T16:00:00Z".to_string()),
            },
            Feature {
                id: 7,
                priority: 40,
                category: "DevOps".to_string(),
                name: "Docker Setup".to_string(),
                description: "Containerize application".to_string(),
                steps: vec![
                    "Create Dockerfile".to_string(),
                    "Setup docker-compose".to_string(),
                ],
                passes: true,
                in_progress: false,
                dependencies: vec![],
                created_at: Some("2024-01-12T09:00:00Z".to_string()),
                updated_at: Some("2024-01-13T11:00:00Z".to_string()),
            },
            Feature {
                id: 8,
                priority: 30,
                category: "Frontend".to_string(),
                name: "Settings Page".to_string(),
                description: "User settings and preferences".to_string(),
                steps: vec![
                    "Design UI".to_string(),
                    "Implement form handling".to_string(),
                ],
                passes: true,
                in_progress: false,
                dependencies: vec![1, 2],
                created_at: Some("2024-01-11T10:00:00Z".to_string()),
                updated_at: Some("2024-01-12T15:00:00Z".to_string()),
            },
            Feature {
                id: 9,
                priority: 20,
                category: "Documentation".to_string(),
                name: "API Documentation".to_string(),
                description: "Complete OpenAPI specs".to_string(),
                steps: vec!["Write specs".to_string(), "Generate docs".to_string()],
                passes: true,
                in_progress: false,
                dependencies: vec![1, 3],
                created_at: Some("2024-01-10T13:00:00Z".to_string()),
                updated_at: Some("2024-01-11T16:00:00Z".to_string()),
            },
            Feature {
                id: 10,
                priority: 10,
                category: "Security".to_string(),
                name: "Security Audit".to_string(),
                description: "Comprehensive security review".to_string(),
                steps: vec![
                    "Run vulnerability scan".to_string(),
                    "Review dependencies".to_string(),
                    "Fix issues".to_string(),
                ],
                passes: true,
                in_progress: false,
                dependencies: vec![1, 3, 6],
                created_at: Some("2024-01-09T08:00:00Z".to_string()),
                updated_at: Some("2024-01-10T18:00:00Z".to_string()),
            },
        ];

        self.add_log("Loaded 10 demo features".to_string());
        self.add_log(format!("Current theme: {}", self.theme.name()));
        self.add_log("Press ? for help".to_string());
    }
}
