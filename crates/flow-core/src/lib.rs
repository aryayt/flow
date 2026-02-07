//! Core types, configuration, and error handling for the Agent Flow workspace.
//!
//! This crate provides the shared foundation used by all other `flow-*` crates:
//! - [`Feature`] and [`Task`] types for work item tracking
//! - [`Config`] for workspace configuration (loaded from TOML)
//! - [`FlowError`] for unified error handling across the workspace
//! - [`Theme`] for terminal color customization
//!
//! # Example
//!
//! ```rust
//! use flow_core::Config;
//!
//! let config = Config::load().unwrap_or_default();
//! println!("Projects dir: {:?}", config.projects_dir);
//! ```

pub mod agent;
pub mod config;
pub mod error;
pub mod event;
pub mod feature;
pub mod project;
pub mod state;
pub mod task;
pub mod theme;

pub use agent::AgentConfig;
pub use config::Config;
pub use error::FlowError;
pub use event::FlowEvent;
pub use feature::{CreateFeatureInput, DependencyRef, Feature, FeatureGraphNode, FeatureStats};
pub use task::{SessionListItem, SessionMeta, Task, TaskWithSession};
pub use theme::{AnsiColor, Theme, ThemeColors};

/// Convenience type alias for Results using `FlowError`.
pub type Result<T> = std::result::Result<T, FlowError>;
