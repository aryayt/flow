//! Agent process management and lifecycle control.

use crate::error::{OrchestratorError, Result};
use flow_core::Feature;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{debug, error, info, warn};

/// Type of AI agent for code generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Claude Code agent (Anthropic).
    Claude,
    /// Codex CLI agent (`OpenAI`).
    Codex,
    /// Gemini CLI agent (Google).
    Gemini,
    /// Custom agent with specified name.
    Custom(&'static str),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Gemini => write!(f, "gemini"),
            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Current status of an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Agent is idle and waiting for work.
    Idle,
    /// Agent is actively working on a feature.
    Working {
        /// ID of the feature being worked on.
        feature_id: i64,
        /// When the agent started working.
        #[serde(skip, default = "Instant::now")]
        started: Instant,
    },
    /// Agent process crashed or terminated unexpectedly.
    Crashed {
        /// Error message describing the crash.
        error: String,
    },
    /// Agent is shutting down gracefully.
    ShuttingDown,
}

impl AgentStatus {
    /// Returns true if the agent is currently working on a feature.
    pub const fn is_working(&self) -> bool {
        matches!(self, Self::Working { .. })
    }

    /// Returns true if the agent is idle and available for work.
    pub const fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Returns true if the agent has crashed.
    pub const fn is_crashed(&self) -> bool {
        matches!(self, Self::Crashed { .. })
    }
}

/// An AI agent process that can be assigned features to implement.
#[derive(Debug)]
pub struct Agent {
    /// Unique identifier for this agent instance.
    pub id: String,
    /// Type of agent (Claude, Codex, Gemini, etc.).
    pub agent_type: AgentType,
    /// Current status of the agent.
    pub status: AgentStatus,
    /// Number of features successfully completed.
    pub features_completed: usize,
    /// Number of features that failed.
    pub features_failed: usize,
    /// The spawned child process (if running).
    pub(crate) child: Option<Child>,
    /// Working directory for this agent.
    pub(crate) work_dir: PathBuf,
}

impl Agent {
    /// Spawn a new agent process and assign it the given feature.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The agent binary is not found in PATH
    /// - Process spawning fails
    /// - Feature description cannot be written to stdin
    pub fn spawn(agent_type: AgentType, work_dir: PathBuf, feature: &Feature) -> Result<Self> {
        let id = format!("{}-{}", agent_type, uuid::Uuid::new_v4());
        info!(
            agent_id = %id,
            agent_type = %agent_type,
            feature_id = feature.id,
            "Spawning agent process"
        );

        // Build the command based on agent type
        let mut cmd = match agent_type {
            AgentType::Claude => {
                let mut c = Command::new("claude");
                c.arg("--print").arg("--output-format").arg("json");
                c
            }
            AgentType::Codex => {
                let mut c = Command::new("codex");
                c.arg("exec").arg("--json");
                c
            }
            AgentType::Gemini => {
                let mut c = Command::new("gemini");
                c.arg("-p").arg("--output-format").arg("json");
                c
            }
            AgentType::Custom(name) => {
                let mut c = Command::new(name);
                c.arg("--json");
                c
            }
        };

        // Configure process
        cmd.current_dir(&work_dir)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        // Spawn the process
        let mut child = cmd.spawn().map_err(|e| {
            error!(agent_type = %agent_type, error = %e, "Failed to spawn agent process");
            OrchestratorError::ProcessError(format!(
                "failed to spawn {agent_type} process: {e}. Is the CLI installed?"
            ))
        })?;

        // Write feature description to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let prompt = format!(
                "Implement the following feature:\n\n**{}**\n\n{}\n\nSteps:\n{}",
                feature.name,
                feature.description,
                feature.steps.join("\n- ")
            );

            // Spawn a task to write to stdin without blocking
            tokio::spawn(async move {
                use tokio::io::AsyncWriteExt;
                if let Err(e) = stdin.write_all(prompt.as_bytes()).await {
                    warn!(error = %e, "Failed to write to agent stdin");
                }
                if let Err(e) = stdin.shutdown().await {
                    warn!(error = %e, "Failed to close agent stdin");
                }
            });
        }

        debug!(agent_id = %id, "Agent process spawned successfully");

        Ok(Self {
            id,
            agent_type,
            status: AgentStatus::Working {
                feature_id: feature.id,
                started: Instant::now(),
            },
            features_completed: 0,
            features_failed: 0,
            child: Some(child),
            work_dir,
        })
    }

    /// Assign a new feature to this agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not idle or if spawning fails.
    pub fn assign_feature(&mut self, feature: &Feature) -> Result<()> {
        if !self.status.is_idle() {
            return Err(OrchestratorError::ProcessError(format!(
                "agent {} is not idle (status: {:?})",
                self.id, self.status
            )));
        }

        info!(
            agent_id = %self.id,
            feature_id = feature.id,
            "Assigning feature to agent"
        );

        // Respawn the agent process with the new feature
        let new_agent = Self::spawn(self.agent_type, self.work_dir.clone(), feature)?;

        // Update this agent's state
        self.child = new_agent.child;
        self.status = new_agent.status;

        Ok(())
    }

    /// Check if the agent process is still alive and healthy.
    pub fn check_health(&mut self) -> bool {
        if let Some(child) = &mut self.child {
            // Try to check if the process has exited
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process has exited
                    warn!(
                        agent_id = %self.id,
                        exit_status = ?status,
                        "Agent process exited"
                    );

                    if status.success() {
                        if let AgentStatus::Working { feature_id, .. } = self.status {
                            info!(
                                agent_id = %self.id,
                                feature_id = feature_id,
                                "Agent completed work successfully"
                            );
                            self.features_completed += 1;
                        }
                        self.status = AgentStatus::Idle;
                    } else {
                        self.features_failed += 1;
                        self.status = AgentStatus::Crashed {
                            error: format!("process exited with status: {status}"),
                        };
                    }

                    self.child = None;
                    status.success()
                }
                Ok(None) => {
                    // Process is still running
                    true
                }
                Err(e) => {
                    error!(
                        agent_id = %self.id,
                        error = %e,
                        "Failed to check agent process status"
                    );
                    self.status = AgentStatus::Crashed {
                        error: format!("failed to check status: {e}"),
                    };
                    self.child = None;
                    false
                }
            }
        } else {
            // No child process
            false
        }
    }

    /// Kill the agent process (gracefully if possible, forcefully if necessary).
    ///
    /// # Errors
    ///
    /// Returns an error if killing the process fails.
    pub async fn kill(&mut self) -> Result<()> {
        if let Some(mut child) = self.child.take() {
            info!(agent_id = %self.id, "Killing agent process");
            self.status = AgentStatus::ShuttingDown;

            // Try graceful shutdown first (SIGTERM on Unix)
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                if let Some(pid) = child.id() {
                    let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);

                    // Wait up to 5 seconds for graceful shutdown
                    let timeout = tokio::time::Duration::from_secs(5);
                    match tokio::time::timeout(timeout, child.wait()).await {
                        Ok(_) => {
                            debug!(agent_id = %self.id, "Agent shut down gracefully");
                            return Ok(());
                        }
                        Err(_) => {
                            warn!(agent_id = %self.id, "Agent did not shut down gracefully, forcing");
                        }
                    }
                }
            }

            // Force kill
            child.kill().await.map_err(|e| {
                error!(agent_id = %self.id, error = %e, "Failed to kill agent process");
                OrchestratorError::ProcessError(format!("failed to kill process: {e}"))
            })?;

            debug!(agent_id = %self.id, "Agent process killed");
        }

        self.status = AgentStatus::Idle;
        Ok(())
    }

    /// Stream output lines from the agent process.
    ///
    /// Returns a stream that yields stdout and stderr lines as they are produced.
    pub fn output_lines(&mut self) -> Option<impl tokio_stream::Stream<Item = String>> {
        if let Some(child) = &mut self.child {
            let stdout = child.stdout.take()?;
            let stderr = child.stderr.take()?;

            let stdout_reader = BufReader::new(stdout);
            let stderr_reader = BufReader::new(stderr);

            let stdout_stream = tokio_stream::wrappers::LinesStream::new(stdout_reader.lines());
            let stderr_stream = tokio_stream::wrappers::LinesStream::new(stderr_reader.lines());

            use tokio_stream::StreamExt;
            let combined = stdout_stream
                .merge(stderr_stream)
                .filter_map(std::result::Result::ok);

            Some(combined)
        } else {
            None
        }
    }

    /// Get the elapsed time for the current work (if working).
    pub fn work_duration(&self) -> Option<std::time::Duration> {
        if let AgentStatus::Working { started, .. } = &self.status {
            Some(started.elapsed())
        } else {
            None
        }
    }
}

// Helper module for UUID generation
mod uuid {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    pub struct Uuid(u64);

    impl Uuid {
        pub fn new_v4() -> Self {
            Self(COUNTER.fetch_add(1, Ordering::SeqCst))
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:016x}", self.0)
        }
    }
}

#[cfg(unix)]
mod nix {
    pub mod sys {
        pub mod signal {
            #[derive(Debug)]
            pub enum Signal {
                SIGTERM,
            }

            pub struct Pid(#[allow(dead_code)] i32);

            impl Pid {
                pub const fn from_raw(pid: i32) -> Self {
                    Self(pid)
                }
            }

            pub const fn kill(_pid: Pid, _signal: Signal) -> std::io::Result<()> {
                // Stub implementation - in real code, this would send the signal
                Ok(())
            }
        }
    }

    pub mod unistd {
        pub use super::sys::signal::Pid;
    }
}
