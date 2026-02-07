//! Terminal UI for Agent Flow with Kanban board and agent status views.
//!
//! Built on [ratatui](https://docs.rs/ratatui) with multiple view modes:
//! - Kanban board for feature status tracking
//! - Agent status dashboard
//! - Dependency graph visualization
//!
//! Integrates with `flow-db` and `flow-resolver` for real-time data.

pub mod app;
pub mod input;
pub mod theme;
pub mod views;
