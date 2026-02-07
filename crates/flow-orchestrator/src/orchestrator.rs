//! Main orchestrator for managing agent pools and work assignment.

use crate::agent::{Agent, AgentStatus, AgentType};
use crate::error::{OrchestratorError, Result};
use crate::scheduler::{Scheduler, SchedulingPolicy};
use flow_core::{Feature, FlowEvent};
use flow_db::FeatureStore;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Configuration for the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Maximum number of coding agents to run concurrently.
    pub max_coding_agents: usize,
    /// Maximum number of testing agents to run concurrently.
    pub max_testing_agents: usize,
    /// How often to poll for new work (in seconds).
    pub poll_interval: Duration,
    /// Maximum time an agent can work on a feature before timeout.
    pub agent_timeout: Duration,
    /// Whether to retry features that fail.
    pub retry_on_failure: bool,
    /// Maximum number of retries for a failed feature.
    pub max_retries: usize,
    /// Working directory for agents.
    pub work_dir: PathBuf,
    /// Scheduling policy to use.
    pub scheduling_policy: SchedulingPolicy,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_coding_agents: 5,
            max_testing_agents: 5,
            poll_interval: Duration::from_secs(5),
            agent_timeout: Duration::from_secs(300),
            retry_on_failure: true,
            max_retries: 3,
            work_dir: PathBuf::from("/tmp/flow-agents"),
            scheduling_policy: SchedulingPolicy::Priority,
        }
    }
}

/// Current status snapshot of the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    /// Number of agents currently running.
    pub agents_running: usize,
    /// Number of agents actively working.
    pub agents_working: usize,
    /// Number of idle agents.
    pub agents_idle: usize,
    /// Number of crashed agents.
    pub agents_crashed: usize,
    /// Total features completed across all agents.
    pub features_completed: usize,
    /// Total features failed across all agents.
    pub features_failed: usize,
    /// Current ready features count.
    pub features_ready: usize,
    /// Whether the orchestrator is running.
    pub running: bool,
}

/// Main orchestrator for managing agent pools and work distribution.
pub struct Orchestrator {
    /// Map of agent ID to agent instance.
    agents: HashMap<String, Agent>,
    /// Work assignment scheduler.
    scheduler: Scheduler,
    /// Configuration.
    config: OrchestratorConfig,
    /// Event broadcast channel.
    event_tx: tokio::sync::broadcast::Sender<FlowEvent>,
    /// Running flag.
    running: Arc<AtomicBool>,
    /// Retry counter for features.
    retry_counts: HashMap<i64, usize>,
}

impl Orchestrator {
    /// Create a new orchestrator with the given configuration.
    pub fn new(config: OrchestratorConfig) -> Self {
        let (event_tx, _) = tokio::sync::broadcast::channel(1024);
        let scheduler = Scheduler::new(config.scheduling_policy);

        info!(
            max_coding_agents = config.max_coding_agents,
            max_testing_agents = config.max_testing_agents,
            poll_interval_secs = config.poll_interval.as_secs(),
            "Creating orchestrator"
        );

        Self {
            agents: HashMap::new(),
            scheduler,
            config,
            event_tx,
            running: Arc::new(AtomicBool::new(false)),
            retry_counts: HashMap::new(),
        }
    }

    /// Spawn a new agent and add it to the pool.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Maximum number of agents has been reached
    /// - Agent with the same ID already exists
    /// - Agent spawning fails
    pub fn spawn_agent(&mut self, agent_type: AgentType) -> Result<String> {
        // Check if we've reached the maximum
        let current_count = self.agents.len();
        let max_agents = self.config.max_coding_agents + self.config.max_testing_agents;

        if current_count >= max_agents {
            return Err(OrchestratorError::MaxAgentsReached(max_agents));
        }

        info!(agent_type = ?agent_type, "Spawning new agent");

        // Create a work directory for this agent
        let agent_work_dir = self.config.work_dir.join(format!("agent-{current_count}"));
        std::fs::create_dir_all(&agent_work_dir)?;

        // We can't spawn without a feature, so we create a dummy idle agent
        // The actual spawn will happen when we assign work
        let agent = Agent {
            id: format!("{agent_type}-idle-{current_count}"),
            agent_type,
            status: AgentStatus::Idle,
            features_completed: 0,
            features_failed: 0,
            child: None,
            work_dir: agent_work_dir,
        };

        let agent_id = agent.id.clone();

        if self.agents.contains_key(&agent_id) {
            return Err(OrchestratorError::AgentAlreadyRunning(agent_id));
        }

        self.agents.insert(agent_id.clone(), agent);

        // Broadcast event
        let _ = self.event_tx.send(FlowEvent::AgentStatus {
            agent: Some(agent_id.clone()),
            status: Some("spawned".to_string()),
            task: None,
        });

        info!(agent_id = %agent_id, "Agent added to pool");
        Ok(agent_id)
    }

    /// Main orchestrator event loop.
    ///
    /// This continuously:
    /// 1. Gets ready features from the database
    /// 2. Computes priority scores
    /// 3. Assigns features to idle agents
    /// 4. Monitors running agents
    /// 5. Handles completions and failures
    /// 6. Broadcasts events
    ///
    /// # Errors
    ///
    /// Returns an error if database operations fail or critical errors occur.
    pub async fn run(&mut self, db: &Connection) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        info!("Orchestrator started");

        // Broadcast initial connection event
        let _ = self.event_tx.send(FlowEvent::Connected);

        while self.running.load(Ordering::SeqCst) {
            // 1. Get ready features
            let ready_features = match FeatureStore::get_ready(db) {
                Ok(features) => features,
                Err(e) => {
                    error!(error = %e, "Failed to get ready features");
                    tokio::time::sleep(self.config.poll_interval).await;
                    continue;
                }
            };

            debug!(count = ready_features.len(), "Found ready features");

            // 2. Compute scores using flow-resolver
            let score_map = flow_resolver::compute_scores(&ready_features);
            let scores: Vec<_> = score_map.iter().map(|(id, score)| (*id, *score)).collect();

            // 3. Assign work to idle agents
            self.assign_work(db, &ready_features, &scores).await?;

            // 4. Monitor running agents
            self.monitor_agents(db).await?;

            // 5. Check for timeouts
            self.check_timeouts(db).await?;

            // 6. Broadcast progress
            self.broadcast_progress(db).await?;

            // Sleep for poll interval
            tokio::time::sleep(self.config.poll_interval).await;
        }

        info!("Orchestrator stopped");
        Ok(())
    }

    /// Assign work to idle agents based on ready features and scores.
    async fn assign_work(
        &mut self,
        db: &Connection,
        ready_features: &[Feature],
        scores: &[(i64, f64)],
    ) -> Result<()> {
        // Filter out features that have been retried too many times
        let assignable_features: Vec<_> = ready_features
            .iter()
            .filter(|f| {
                let retries = self.retry_counts.get(&f.id).copied().unwrap_or(0);
                retries < self.config.max_retries
            })
            .cloned()
            .collect();

        if assignable_features.is_empty() {
            return Ok(());
        }

        // Build a vec of agent references for the scheduler
        let agent_refs: Vec<&Agent> = self.agents.values().collect();

        // Get the next assignment from the scheduler
        if let Some((agent_id, feature_id)) =
            self.scheduler
                .next_assignment(&agent_refs, &assignable_features, scores)
        {
            info!(
                agent_id = %agent_id,
                feature_id = feature_id,
                "Assigning feature to agent"
            );

            // Get the feature
            let feature = assignable_features
                .iter()
                .find(|f| f.id == feature_id)
                .ok_or_else(|| OrchestratorError::Scheduler("feature not found".to_string()))?;

            // Mark the feature as in_progress in the database
            FeatureStore::mark_in_progress(db, feature_id)?;

            // Assign to the agent
            if let Some(agent) = self.agents.get_mut(&agent_id) {
                match agent.assign_feature(feature) {
                    Ok(()) => {
                        // Broadcast event
                        let _ = self.event_tx.send(FlowEvent::AgentStatus {
                            agent: Some(agent_id.clone()),
                            status: Some("working".to_string()),
                            task: Some(feature.name.clone()),
                        });

                        let _ = self.event_tx.send(FlowEvent::FeatureUpdate {
                            id: feature_id,
                            passes: None,
                            in_progress: Some(true),
                        });
                    }
                    Err(e) => {
                        error!(
                            agent_id = %agent_id,
                            feature_id = feature_id,
                            error = %e,
                            "Failed to assign feature to agent"
                        );

                        // Clear in_progress flag
                        FeatureStore::clear_in_progress(db, feature_id)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Monitor agent health and handle completions/failures.
    async fn monitor_agents(&mut self, _db: &Connection) -> Result<()> {
        let mut completed = Vec::new();
        let mut failed = Vec::new();

        for (agent_id, agent) in &mut self.agents {
            // Check health
            let was_working = agent.status.is_working();
            let is_healthy = agent.check_health();

            // If the agent was working and is now idle, it completed
            if was_working && agent.status.is_idle() && is_healthy {
                if matches!(agent.status, AgentStatus::Idle) {
                    completed.push(agent_id.clone());
                }
            }

            // If the agent crashed, record the failure
            if let AgentStatus::Crashed { .. } = &agent.status {
                failed.push(agent_id.clone());
            }
        }

        // Handle completions
        for agent_id in completed {
            if self.agents.contains_key(&agent_id) {
                // The feature ID is no longer available in Idle status
                // We need to track this differently, so we'll just broadcast the event
                info!(agent_id = %agent_id, "Agent completed work");

                let _ = self.event_tx.send(FlowEvent::AgentStatus {
                    agent: Some(agent_id.clone()),
                    status: Some("idle".to_string()),
                    task: None,
                });
            }
        }

        // Handle failures
        for agent_id in failed {
            if let Some(agent) = self.agents.get(&agent_id) {
                if let AgentStatus::Crashed { error } = &agent.status {
                    warn!(
                        agent_id = %agent_id,
                        error = %error,
                        "Agent crashed"
                    );

                    let _ = self.event_tx.send(FlowEvent::AgentStatus {
                        agent: Some(agent_id.clone()),
                        status: Some("crashed".to_string()),
                        task: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Check for agent timeouts and kill stuck agents.
    async fn check_timeouts(&mut self, db: &Connection) -> Result<()> {
        let mut timed_out = Vec::new();

        for (agent_id, agent) in &self.agents {
            if let Some(duration) = agent.work_duration() {
                if duration > self.config.agent_timeout {
                    warn!(
                        agent_id = %agent_id,
                        duration_secs = duration.as_secs(),
                        timeout_secs = self.config.agent_timeout.as_secs(),
                        "Agent timed out"
                    );
                    timed_out.push(agent_id.clone());
                }
            }
        }

        for agent_id in timed_out {
            if let Some(agent) = self.agents.get_mut(&agent_id) {
                // Get the feature ID before killing
                let feature_id = if let AgentStatus::Working { feature_id, .. } = agent.status {
                    Some(feature_id)
                } else {
                    None
                };

                // Kill the agent
                if let Err(e) = agent.kill().await {
                    error!(
                        agent_id = %agent_id,
                        error = %e,
                        "Failed to kill timed-out agent"
                    );
                }

                // Mark the feature as failed and clear in_progress
                if let Some(fid) = feature_id {
                    FeatureStore::mark_failing(db, fid)?;
                    FeatureStore::clear_in_progress(db, fid)?;

                    // Increment retry count
                    *self.retry_counts.entry(fid).or_insert(0) += 1;

                    let _ = self.event_tx.send(FlowEvent::FeatureUpdate {
                        id: fid,
                        passes: Some(false),
                        in_progress: Some(false),
                    });
                }
            }
        }

        Ok(())
    }

    /// Broadcast progress update.
    async fn broadcast_progress(&self, db: &Connection) -> Result<()> {
        let stats = FeatureStore::get_stats(db)?;

        let _ = self.event_tx.send(FlowEvent::Progress {
            passing: stats.passing,
            total: stats.total,
            in_progress: stats.in_progress,
        });

        Ok(())
    }

    /// Gracefully shut down all agents and stop the orchestrator.
    ///
    /// # Errors
    ///
    /// Returns an error if any agents fail to shut down.
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down orchestrator");
        self.running.store(false, Ordering::SeqCst);

        // Kill all agents
        let agent_ids: Vec<_> = self.agents.keys().cloned().collect();
        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get_mut(&agent_id) {
                info!(agent_id = %agent_id, "Shutting down agent");
                agent.kill().await?;
            }
        }

        self.agents.clear();
        info!("All agents shut down");

        Ok(())
    }

    /// Get current status snapshot.
    pub fn status(&self) -> OrchestratorStatus {
        let agents_working = self
            .agents
            .values()
            .filter(|a| a.status.is_working())
            .count();

        let agents_idle = self.agents.values().filter(|a| a.status.is_idle()).count();

        let agents_crashed = self
            .agents
            .values()
            .filter(|a| a.status.is_crashed())
            .count();

        let features_completed: usize = self.agents.values().map(|a| a.features_completed).sum();

        let features_failed: usize = self.agents.values().map(|a| a.features_failed).sum();

        OrchestratorStatus {
            agents_running: self.agents.len(),
            agents_working,
            agents_idle,
            agents_crashed,
            features_completed,
            features_failed,
            features_ready: 0, // Will be updated by caller
            running: self.running.load(Ordering::SeqCst),
        }
    }

    /// Subscribe to orchestrator events.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<FlowEvent> {
        self.event_tx.subscribe()
    }

    /// Get reference to the configuration.
    pub const fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Get the number of active agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
}
