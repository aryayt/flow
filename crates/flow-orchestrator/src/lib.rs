//! Agent orchestration system for Flow.
//!
//! This crate provides the core orchestration logic for managing multiple AI coding agents
//! (Claude, Codex, Gemini) working on features in parallel. It handles:
//!
//! - **Agent process management**: Spawning, monitoring, and terminating agent processes
//! - **Work scheduling**: Assigning features to agents based on priority, round-robin, or specialty matching
//! - **Health monitoring**: Detecting crashes, timeouts, and handling failures
//! - **Task synchronization**: Watching Claude Code task files for real-time updates
//! - **Event broadcasting**: Publishing orchestrator events via tokio broadcast channels
//!
//! # Architecture
//!
//! The orchestrator follows a simple event loop pattern:
//!
//! 1. **Poll for ready features** from the database (features with all dependencies met)
//! 2. **Compute priority scores** using the flow-resolver dependency graph analysis
//! 3. **Assign work** to idle agents using the configured scheduling policy
//! 4. **Monitor agents** for completion, crashes, or timeouts
//! 5. **Broadcast events** for UI updates and logging
//! 6. **Sleep** for the configured poll interval, then repeat
//!
//! # Example
//!
//! ```rust,no_run
//! use flow_orchestrator::{Orchestrator, OrchestratorConfig, AgentType};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create orchestrator
//! let config = OrchestratorConfig {
//!     max_coding_agents: 3,
//!     work_dir: PathBuf::from("/tmp/agents"),
//!     ..Default::default()
//! };
//!
//! let mut orchestrator = Orchestrator::new(config);
//!
//! // Spawn agents
//! orchestrator.spawn_agent(AgentType::Claude)?;
//! orchestrator.spawn_agent(AgentType::Codex)?;
//!
//! // Open database
//! let db = flow_db::Database::open(std::path::Path::new("flow.db"))?;
//! let conn = db.writer().lock().unwrap();
//!
//! // Run orchestrator (blocks until shutdown)
//! orchestrator.run(&conn).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Scheduling Policies
//!
//! The orchestrator supports three scheduling policies:
//!
//! - **Priority**: Assign highest-scoring features first (based on dependency graph)
//! - **`RoundRobin`**: Distribute work evenly across all agents
//! - **`SpecialtyMatch`**: Match agent types to feature categories (e.g., Gemini for testing)
//!
//! # Task Synchronization
//!
//! The [`TaskWatcher`] monitors `~/.claude/tasks/` for changes and broadcasts
//! [`FlowEvent::TaskUpdate`] events when task JSON files are created, modified, or deleted.
//! This enables real-time synchronization between Claude Code's native task system and
//! the Flow feature database.

pub mod agent;
pub mod error;
pub mod orchestrator;
pub mod scheduler;
pub mod watcher;

pub use agent::{Agent, AgentStatus, AgentType};
pub use error::{OrchestratorError, Result};
pub use orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorStatus};
pub use scheduler::{Assignment, Scheduler, SchedulingPolicy};
pub use watcher::TaskWatcher;
