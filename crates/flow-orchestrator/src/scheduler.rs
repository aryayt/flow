//! Work assignment and scheduling logic for agents.

use crate::agent::{Agent, AgentType};
use flow_core::Feature;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Scheduling policy for assigning features to agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingPolicy {
    /// Assign highest priority features first, regardless of agent specialty.
    Priority,
    /// Distribute work evenly across agents in round-robin fashion.
    RoundRobin,
    /// Match agent specialties to feature categories when possible.
    SpecialtyMatch,
}

impl Default for SchedulingPolicy {
    fn default() -> Self {
        Self::Priority
    }
}

/// Record of a feature assignment to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    /// ID of the agent that was assigned the feature.
    pub agent_id: String,
    /// ID of the feature that was assigned.
    pub feature_id: i64,
    /// When the assignment was made.
    pub assigned_at: chrono::DateTime<chrono::Utc>,
}

/// Scheduler for assigning features to agents based on a policy.
#[derive(Debug)]
pub struct Scheduler {
    /// The scheduling policy to use.
    policy: SchedulingPolicy,
    /// History of assignments for round-robin tracking.
    assignment_history: Vec<Assignment>,
}

impl Scheduler {
    /// Create a new scheduler with the given policy.
    pub const fn new(policy: SchedulingPolicy) -> Self {
        Self {
            policy,
            assignment_history: Vec::new(),
        }
    }

    /// Select the next agent-feature assignment based on the scheduling policy.
    ///
    /// # Arguments
    ///
    /// * `agents` - Available agents (both idle and working)
    /// * `ready_features` - Features that are ready to be assigned
    /// * `scores` - Priority scores for each feature (from flow-resolver)
    ///
    /// # Returns
    ///
    /// Returns `Some((agent_id, feature_id))` if an assignment can be made, or `None` if
    /// no idle agents are available or no features are ready.
    pub fn next_assignment(
        &mut self,
        agents: &[&Agent],
        ready_features: &[Feature],
        scores: &[(i64, f64)],
    ) -> Option<(String, i64)> {
        if ready_features.is_empty() {
            debug!("No ready features to assign");
            return None;
        }

        // Filter to idle agents only (agents is already &[&Agent], so we don't need to re-reference)
        let idle_agents: Vec<_> = agents
            .iter()
            .copied()
            .filter(|a| a.status.is_idle())
            .collect();

        if idle_agents.is_empty() {
            debug!("No idle agents available for assignment");
            return None;
        }

        // Create a score lookup map for quick access
        let score_map: std::collections::HashMap<i64, f64> = scores.iter().copied().collect();

        let assignment = match self.policy {
            SchedulingPolicy::Priority => {
                self.schedule_by_priority(&idle_agents, ready_features, &score_map)
            }
            SchedulingPolicy::RoundRobin => self.schedule_round_robin(&idle_agents, ready_features),
            SchedulingPolicy::SpecialtyMatch => {
                self.schedule_by_specialty(&idle_agents, ready_features, &score_map)
            }
        };

        if let Some((agent_id, feature_id)) = &assignment {
            // Record the assignment
            self.assignment_history.push(Assignment {
                agent_id: agent_id.clone(),
                feature_id: *feature_id,
                assigned_at: chrono::Utc::now(),
            });

            info!(
                agent_id = %agent_id,
                feature_id = %feature_id,
                policy = ?self.policy,
                "Scheduled assignment"
            );
        }

        assignment
    }

    /// Schedule by priority - assign highest-scoring feature to first idle agent.
    #[allow(clippy::unused_self)]
    fn schedule_by_priority(
        &self,
        idle_agents: &[&Agent],
        ready_features: &[Feature],
        score_map: &std::collections::HashMap<i64, f64>,
    ) -> Option<(String, i64)> {
        // Sort features by score (descending)
        let mut scored_features: Vec<_> = ready_features
            .iter()
            .map(|f| (f, score_map.get(&f.id).copied().unwrap_or(0.0)))
            .collect();

        scored_features.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Assign the highest-scoring feature to the first idle agent
        if let Some((feature, score)) = scored_features.first() {
            if let Some(agent) = idle_agents.first() {
                debug!(
                    agent_id = %agent.id,
                    feature_id = feature.id,
                    score = score,
                    "Priority scheduling: assigning highest-scoring feature"
                );
                return Some((agent.id.clone(), feature.id));
            }
        }

        None
    }

    /// Schedule round-robin - distribute work evenly across agents.
    fn schedule_round_robin(
        &self,
        idle_agents: &[&Agent],
        ready_features: &[Feature],
    ) -> Option<(String, i64)> {
        if idle_agents.is_empty() || ready_features.is_empty() {
            return None;
        }

        // Count assignments per agent
        let mut agent_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for assignment in &self.assignment_history {
            *agent_counts.entry(assignment.agent_id.clone()).or_insert(0) += 1;
        }

        // Find the idle agent with the fewest assignments
        let agent = idle_agents
            .iter()
            .min_by_key(|a| agent_counts.get(&a.id).copied().unwrap_or(0))?;

        // Assign the first ready feature
        let feature = ready_features.first()?;

        debug!(
            agent_id = %agent.id,
            feature_id = feature.id,
            assignments_count = agent_counts.get(&agent.id).copied().unwrap_or(0),
            "Round-robin scheduling"
        );

        Some((agent.id.clone(), feature.id))
    }

    /// Schedule by specialty - match agent types to feature categories.
    fn schedule_by_specialty(
        &self,
        idle_agents: &[&Agent],
        ready_features: &[Feature],
        score_map: &std::collections::HashMap<i64, f64>,
    ) -> Option<(String, i64)> {
        // Build specialty preferences
        let specialty_map = Self::build_specialty_map();

        // Try to find matches between agent types and feature categories
        for feature in ready_features {
            let preferred_types = specialty_map
                .get(feature.category.as_str())
                .copied()
                .unwrap_or(&[]);

            // First try to find an agent with matching specialty
            for preferred_type in preferred_types {
                if let Some(agent) = idle_agents.iter().find(|a| a.agent_type == *preferred_type) {
                    debug!(
                        agent_id = %agent.id,
                        feature_id = feature.id,
                        category = %feature.category,
                        agent_type = ?agent.agent_type,
                        "Specialty match found"
                    );
                    return Some((agent.id.clone(), feature.id));
                }
            }
        }

        // Fallback to priority scheduling if no specialty match found
        debug!("No specialty match found, falling back to priority scheduling");
        self.schedule_by_priority(idle_agents, ready_features, score_map)
    }

    /// Build a map of feature categories to preferred agent types.
    fn build_specialty_map() -> std::collections::HashMap<&'static str, &'static [AgentType]> {
        let mut map = std::collections::HashMap::new();

        // Frontend work: prefer Claude (strong at React, TypeScript)
        map.insert(
            "frontend",
            &[AgentType::Claude, AgentType::Codex] as &[AgentType],
        );
        map.insert("ui", &[AgentType::Claude, AgentType::Codex] as &[AgentType]);

        // Backend work: prefer Codex (strong at Python, Node.js)
        map.insert(
            "backend",
            &[AgentType::Codex, AgentType::Claude] as &[AgentType],
        );
        map.insert(
            "api",
            &[AgentType::Codex, AgentType::Claude] as &[AgentType],
        );

        // Testing: prefer Gemini (good at test generation)
        map.insert(
            "testing",
            &[AgentType::Gemini, AgentType::Codex] as &[AgentType],
        );
        map.insert(
            "test",
            &[AgentType::Gemini, AgentType::Codex] as &[AgentType],
        );

        // Documentation: prefer Gemini (large context window)
        map.insert(
            "docs",
            &[AgentType::Gemini, AgentType::Claude] as &[AgentType],
        );
        map.insert(
            "documentation",
            &[AgentType::Gemini, AgentType::Claude] as &[AgentType],
        );

        // Database: prefer Codex
        map.insert(
            "database",
            &[AgentType::Codex, AgentType::Claude] as &[AgentType],
        );
        map.insert("db", &[AgentType::Codex, AgentType::Claude] as &[AgentType]);

        map
    }

    /// Get the assignment history.
    pub fn assignments(&self) -> &[Assignment] {
        &self.assignment_history
    }

    /// Get the current scheduling policy.
    pub const fn policy(&self) -> SchedulingPolicy {
        self.policy
    }

    /// Set a new scheduling policy.
    pub fn set_policy(&mut self, policy: SchedulingPolicy) {
        info!(old_policy = ?self.policy, new_policy = ?policy, "Changing scheduling policy");
        self.policy = policy;
    }

    /// Clear the assignment history (useful for testing or reset).
    pub fn clear_history(&mut self) {
        debug!(
            count = self.assignment_history.len(),
            "Clearing assignment history"
        );
        self.assignment_history.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::AgentStatus;

    fn mock_agent(id: &str, agent_type: AgentType, status: AgentStatus) -> Agent {
        Agent {
            id: id.to_string(),
            agent_type,
            status,
            features_completed: 0,
            features_failed: 0,
            child: None,
            work_dir: std::path::PathBuf::from("/tmp"),
        }
    }

    fn mock_feature(id: i64, category: &str) -> Feature {
        Feature {
            id,
            priority: 1,
            category: category.to_string(),
            name: format!("Feature {id}"),
            description: String::new(),
            steps: vec![],
            passes: false,
            in_progress: false,
            dependencies: vec![],
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_priority_scheduling() {
        let mut scheduler = Scheduler::new(SchedulingPolicy::Priority);

        let agents = vec![
            mock_agent("agent-1", AgentType::Claude, AgentStatus::Idle),
            mock_agent("agent-2", AgentType::Codex, AgentStatus::Idle),
        ];

        let features = vec![mock_feature(1, "frontend"), mock_feature(2, "backend")];

        let scores = vec![(1, 10.0), (2, 5.0)];

        let agent_refs: Vec<&Agent> = agents.iter().collect();
        let assignment = scheduler.next_assignment(&agent_refs, &features, &scores);
        assert!(assignment.is_some());

        let (agent_id, feature_id) = assignment.unwrap();
        assert_eq!(feature_id, 1); // Highest score
        assert_eq!(agent_id, "agent-1"); // First idle agent
    }

    #[test]
    fn test_round_robin() {
        let mut scheduler = Scheduler::new(SchedulingPolicy::RoundRobin);

        let agents = vec![
            mock_agent("agent-1", AgentType::Claude, AgentStatus::Idle),
            mock_agent("agent-2", AgentType::Codex, AgentStatus::Idle),
        ];

        let features = vec![mock_feature(1, "frontend"), mock_feature(2, "backend")];

        let agent_refs: Vec<&Agent> = agents.iter().collect();

        // First assignment
        let assignment1 = scheduler.next_assignment(&agent_refs, &features, &[]);
        assert_eq!(assignment1, Some(("agent-1".to_string(), 1)));

        // Second assignment should go to agent-2 (round-robin)
        let assignment2 = scheduler.next_assignment(&agent_refs, &features, &[]);
        assert_eq!(assignment2, Some(("agent-2".to_string(), 1)));
    }

    #[test]
    fn test_specialty_match() {
        let mut scheduler = Scheduler::new(SchedulingPolicy::SpecialtyMatch);

        let agents = vec![
            mock_agent("agent-1", AgentType::Claude, AgentStatus::Idle),
            mock_agent("agent-2", AgentType::Gemini, AgentStatus::Idle),
        ];

        let features = vec![
            mock_feature(1, "testing"), // Prefers Gemini
        ];

        let agent_refs: Vec<&Agent> = agents.iter().collect();
        let assignment = scheduler.next_assignment(&agent_refs, &features, &[]);
        assert!(assignment.is_some());

        let (agent_id, _) = assignment.unwrap();
        assert_eq!(agent_id, "agent-2"); // Gemini preferred for testing
    }

    #[test]
    fn test_no_idle_agents() {
        let mut scheduler = Scheduler::new(SchedulingPolicy::Priority);

        let agents = vec![mock_agent(
            "agent-1",
            AgentType::Claude,
            AgentStatus::Working {
                feature_id: 1,
                started: std::time::Instant::now(),
            },
        )];

        let features = vec![mock_feature(1, "frontend")];

        let agent_refs: Vec<&Agent> = agents.iter().collect();
        let assignment = scheduler.next_assignment(&agent_refs, &features, &[]);
        assert!(assignment.is_none());
    }
}
