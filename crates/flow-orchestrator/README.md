# flow-orchestrator

Multi-agent process management, batch scheduling, and collaborative workflow orchestration.

## What it does

`flow-orchestrator` is like a conductor leading an orchestra of AI agents. Instead of manually telling each agent what to do, it:

- **Assigns tasks** to multiple agents based on their specialties
- **Prevents conflicts** when agents try to work on the same feature
- **Optimizes throughput** by scheduling work efficiently
- **Handles failures** by reassigning features when agents crash
- **Coordinates handoffs** when one feature depends on another

Think of it as an automated project manager that ensures multiple AI assistants can work together without stepping on each other's toes.

## Status

⚠️ **Phase 5 - Planned for Future Implementation**

This crate is currently a placeholder. Full implementation is scheduled for Phase 5 of the Flow project, after the MCP server is complete.

## Planned Architecture

```
flow-orchestrator/
├── lib.rs              - Public API and orchestrator setup
├── agent.rs            - Agent process management
├── scheduler.rs        - Work assignment algorithm
├── coordinator.rs      - Inter-agent communication
├── monitor.rs          - Health checking and failure detection
├── batch.rs            - Batch job scheduling
├── queue.rs            - Shared work queue
├── lock.rs             - Distributed locking
└── config.rs           - Agent configuration and capabilities
```

### System Architecture

```
┌─────────────────────────────────────────────────┐
│              Flow Orchestrator                   │
│                                                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │Scheduler │─>│  Queue   │<─│ Monitor  │       │
│  └──────────┘  └──────────┘  └──────────┘       │
│       │             │              │             │
├───────┼─────────────┼──────────────┼─────────────┤
│       ↓             ↓              ↓             │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐          │
│  │Agent 1  │  │Agent 2  │  │Agent 3  │          │
│  │(Claude) │  │(Codex)  │  │(Gemini) │          │
│  └─────────┘  └─────────┘  └─────────┘          │
│       │             │              │             │
├───────┼─────────────┼──────────────┼─────────────┤
│       ↓             ↓              ↓             │
│               Flow Database                      │
└─────────────────────────────────────────────────┘
```

## Planned Features

### Agent Management

- **Process Spawning**: Launch agent processes with custom configurations
- **Health Monitoring**: Detect crashed or hung agents
- **Auto-Restart**: Automatically restart failed agents
- **Resource Limits**: CPU/memory constraints per agent
- **Graceful Shutdown**: Clean termination of all agents

### Scheduling Algorithms

- **Priority-Based**: Work on highest-score features first
- **Round-Robin**: Fair distribution across agents
- **Specialty Matching**: Assign backend work to backend experts
- **Load Balancing**: Even distribution of work
- **Dependency-Aware**: Respect feature dependencies

### Coordination

- **Distributed Locking**: Prevent concurrent work on same feature
- **Handoff Protocol**: Pass completed features to dependents
- **Conflict Resolution**: Handle merge conflicts automatically
- **Status Broadcasting**: Real-time agent status updates
- **Chat/Messaging**: Inter-agent communication

## Expected Usage

### Starting the Orchestrator

```bash
# Start with default agents
flow-orchestrator --agents claude,codex,gemini

# Custom configuration
flow-orchestrator --config agents.toml

# Batch mode (process all features, then exit)
flow-orchestrator --batch --max-concurrent 3
```

### Configuration File

```toml
# agents.toml

[orchestrator]
max_concurrent_agents = 3
scheduler_algorithm = "priority"
health_check_interval = 10  # seconds
work_timeout = 300           # 5 minutes per feature

[[agents]]
name = "claude"
specialty = ["backend", "architecture"]
max_features = 5
command = "claude-cli"
args = ["--mode", "autonomous"]

[[agents]]
name = "codex"
specialty = ["frontend", "ui"]
max_features = 3
command = "codex"
args = ["--non-interactive"]

[[agents]]
name = "gemini"
specialty = ["testing", "documentation"]
max_features = 2
command = "gemini"
args = ["--batch"]
```

### Programmatic API

```rust
use flow_orchestrator::{Orchestrator, AgentConfig, SchedulingPolicy};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure orchestrator
    let orchestrator = Orchestrator::builder()
        .database_path("features.db")
        .scheduling_policy(SchedulingPolicy::Priority)
        .max_concurrent_agents(3)
        .health_check_interval(Duration::from_secs(10))
        .build()?;

    // Register agents
    orchestrator.register_agent(AgentConfig {
        name: "claude".to_string(),
        command: "claude-cli".to_string(),
        args: vec!["--autonomous".to_string()],
        specialty: vec!["backend".to_string()],
        max_concurrent_features: 5,
    }).await?;

    orchestrator.register_agent(AgentConfig {
        name: "codex".to_string(),
        command: "codex".to_string(),
        args: vec!["--non-interactive".to_string()],
        specialty: vec!["frontend".to_string()],
        max_concurrent_features: 3,
    }).await?;

    // Start orchestration
    orchestrator.start().await?;

    // Wait for completion (batch mode)
    orchestrator.wait_for_completion().await?;

    // Get stats
    let stats = orchestrator.get_statistics().await?;
    println!("Completed: {} features", stats.completed);
    println!("Failed: {} features", stats.failed);
    println!("Average time: {:.1}s", stats.avg_time_per_feature);

    Ok(())
}
```

### Real-time Monitoring

```rust
use flow_orchestrator::Orchestrator;

let orchestrator = Orchestrator::new()?;

// Subscribe to events
let mut events = orchestrator.event_stream();

while let Some(event) = events.next().await {
    match event {
        Event::AgentStarted { name, pid } => {
            println!("Agent {name} started (PID: {pid})");
        }
        Event::FeatureAssigned { agent, feature_id } => {
            println!("Agent {agent} assigned feature #{feature_id}");
        }
        Event::FeatureCompleted { agent, feature_id, duration } => {
            println!("Agent {agent} completed feature #{feature_id} in {duration:.1}s");
        }
        Event::AgentFailed { name, reason } => {
            println!("Agent {name} failed: {reason}");
        }
        Event::AllComplete => {
            println!("All features completed!");
            break;
        }
    }
}
```

## Scheduling Algorithm

### Priority-Based Scheduling

```rust
pub async fn assign_work(&mut self) -> Result<()> {
    // Get available agents
    let available_agents = self.agents.iter()
        .filter(|a| a.current_load() < a.max_concurrent_features)
        .collect::<Vec<_>>();

    if available_agents.is_empty() {
        return Ok(()); // All agents busy
    }

    // Get ready features
    let conn = self.db.writer().lock().unwrap();
    let ready = FeatureStore::get_ready(&conn)?;

    if ready.is_empty() {
        return Ok(()); // No work available
    }

    // Compute scores
    let all_features = FeatureStore::get_all(&conn)?;
    let scores = compute_scores(&all_features);

    // Sort by score (highest first)
    let mut scored_ready: Vec<_> = ready.iter()
        .map(|f| (f, scores[&f.id]))
        .collect();
    scored_ready.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Assign to agents
    for (feature, _score) in scored_ready {
        // Find best agent for this feature
        let agent = self.find_best_agent(feature, &available_agents)?;

        // Claim feature and assign
        let claimed = FeatureStore::claim_and_get(&conn, feature.id)?;
        agent.assign_feature(claimed).await?;

        println!("Assigned {} to {}", feature.name, agent.name);

        // Update availability
        if agent.current_load() >= agent.max_concurrent_features {
            available_agents.retain(|a| a.name != agent.name);
            if available_agents.is_empty() {
                break;
            }
        }
    }

    Ok(())
}

fn find_best_agent(&self, feature: &Feature, agents: &[&Agent]) -> Result<&Agent> {
    // Match by specialty
    for agent in agents {
        if agent.specialty.contains(&feature.category) {
            return Ok(agent);
        }
    }

    // Fallback: least loaded agent
    agents.iter()
        .min_by_key(|a| a.current_load())
        .ok_or_else(|| FlowError::NoAvailableAgent)
        .copied()
}
```

### Batch Scheduling

```rust
pub async fn run_batch(&mut self, max_concurrent: usize) -> Result<BatchStats> {
    let start = Instant::now();
    let mut stats = BatchStats::default();

    loop {
        // Assign work
        self.assign_work().await?;

        // Wait for features to complete
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check status
        let conn = self.db.writer().lock().unwrap();
        let ready = FeatureStore::get_ready(&conn)?;
        let in_progress = FeatureStore::get_all(&conn)?
            .into_iter()
            .filter(|f| f.in_progress)
            .count();

        if ready.is_empty() && in_progress == 0 {
            break; // All done
        }
    }

    stats.total_time = start.elapsed();
    stats.completed = FeatureStore::get_stats(&self.db)?.passing;

    Ok(stats)
}
```

## Agent Process Management

### Spawning Agents

```rust
pub struct Agent {
    name: String,
    process: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    current_features: Vec<i64>,
    max_concurrent: usize,
}

impl Agent {
    pub async fn spawn(config: &AgentConfig) -> Result<Self> {
        let mut child = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        Ok(Self {
            name: config.name.clone(),
            process: child,
            stdin,
            stdout,
            current_features: Vec::new(),
            max_concurrent: config.max_concurrent_features,
        })
    }

    pub async fn assign_feature(&mut self, feature: Feature) -> Result<()> {
        // Send feature to agent via JSON-RPC or stdin
        let command = serde_json::json!({
            "type": "work_on_feature",
            "feature": feature,
        });

        self.stdin.write_all(command.to_string().as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        self.current_features.push(feature.id);

        Ok(())
    }

    pub fn current_load(&self) -> usize {
        self.current_features.len()
    }

    pub async fn health_check(&mut self) -> Result<bool> {
        // Check if process is alive
        match self.process.try_wait()? {
            None => Ok(true), // Still running
            Some(status) => {
                println!("Agent {} exited with status: {}", self.name, status);
                Ok(false)
            }
        }
    }

    pub async fn kill(&mut self) -> Result<()> {
        self.process.kill().await?;
        Ok(())
    }
}
```

### Health Monitoring

```rust
pub async fn monitor_agents(&mut self) -> Result<()> {
    let mut interval = tokio::time::interval(self.health_check_interval);

    loop {
        interval.tick().await;

        for agent in &mut self.agents {
            if !agent.health_check().await? {
                // Agent died, reassign its features
                println!("Agent {} died, reassigning work", agent.name);

                for feature_id in &agent.current_features {
                    FeatureStore::clear_in_progress(&self.db, *feature_id)?;
                }

                // Restart agent
                *agent = Agent::spawn(&agent.config).await?;
            }
        }
    }
}
```

## Distributed Locking

```rust
pub struct FeatureLock {
    feature_id: i64,
    agent_name: String,
    acquired_at: Instant,
}

impl Orchestrator {
    pub async fn try_lock(&self, feature_id: i64, agent: &str) -> Result<bool> {
        let conn = self.db.writer().lock().unwrap();

        // Atomic claim operation
        match FeatureStore::claim_and_get(&conn, feature_id) {
            Ok(_) => {
                self.locks.insert(feature_id, FeatureLock {
                    feature_id,
                    agent_name: agent.to_string(),
                    acquired_at: Instant::now(),
                });
                Ok(true)
            }
            Err(FlowError::Conflict(_)) => {
                Ok(false) // Already locked
            }
            Err(e) => Err(e),
        }
    }

    pub async fn unlock(&self, feature_id: i64) -> Result<()> {
        let conn = self.db.writer().lock().unwrap();
        FeatureStore::clear_in_progress(&conn, feature_id)?;
        self.locks.remove(&feature_id);
        Ok(())
    }
}
```

## Testing

```bash
# Unit tests (when implemented)
cargo test -p flow-orchestrator

# Integration test with mock agents
cargo test -p flow-orchestrator --test integration

# Benchmark scheduling algorithm
cargo bench -p flow-orchestrator
```

## Performance Goals

- **Startup time**: <100ms to spawn all agents
- **Assignment latency**: <10ms to assign a feature
- **Monitoring overhead**: <1% CPU
- **Throughput**: 100+ features/hour with 3 agents

## Related Crates

- **[flow-core](../flow-core/README.md)**: Core types and configuration
- **[flow-db](../flow-db/README.md)**: Database for atomic feature claiming
- **[flow-resolver](../flow-resolver/README.md)**: Priority scoring for scheduling
- **[flow-mcp](../flow-mcp/README.md)**: Agent communication protocol
- **[flow-server](../flow-server/README.md)**: Web dashboard for monitoring

## Future Enhancements

- **Dynamic scaling**: Add/remove agents based on workload
- **Cloud deployment**: Run agents on different machines
- **Kubernetes integration**: Deploy as K8s Jobs
- **Cost optimization**: Track agent API costs and optimize
- **Learning**: Improve agent assignment based on past performance

[Back to main README](../../README.md)
