# flow-core

Shared types, configuration, themes, errors, and events used across all Flow crates.

## What it does

`flow-core` is the foundation crate that every other crate depends on. It defines the common language that all parts of Flow use to communicate. Think of it as the dictionary and grammar rules that everyone agrees on.

It contains:
- **Types**: Feature, Task, Session - the data structures everything works with
- **Configuration**: Both workflow config (`~/.config/flow/config.toml`) and agent monitoring config
- **Themes**: 7 color themes with ANSI (terminal) and CSS (web) representations
- **Errors**: A unified error type with 14 variants for everything that can go wrong
- **Events**: Event types for real-time updates between components

## Architecture

```
flow-core/
├── lib.rs        - Module exports and Result type alias
├── config.rs     - Workflow config (projects_dir, state_dir, default_branch)
├── agent.rs      - AgentConfig (port, theme, concurrency, paths)
├── feature.rs    - Feature, FeatureStats, FeatureGraphNode, CreateFeatureInput
├── task.rs       - Task, SessionListItem, SessionMeta, TaskWithSession
├── theme.rs      - 7 themes with ANSI + CSS color definitions
├── event.rs      - FlowEvent enum for SSE/WebSocket broadcasting
├── error.rs      - FlowError with 14 variants
├── state.rs      - Shared state types
└── project.rs    - Project metadata types
```

## Key Types

### Feature (SQLite-backed)

```rust
pub struct Feature {
    pub id: i64,
    pub priority: i32,       // Lower = more important
    pub category: String,    // e.g., "Backend", "Frontend"
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,  // Implementation checklist
    pub passes: bool,        // Test status
    pub in_progress: bool,   // Currently claimed by an agent
    pub dependencies: Vec<i64>, // Feature IDs this depends on
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
```

### Task (JSON file-based)

```rust
pub struct Task {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub active_form: String,  // What the agent is currently doing
    pub status: String,       // "pending", "in_progress", "completed"
    pub owner: String,        // "claude", "codex", "gemini"
    pub blocks: Vec<String>,
    pub blocked_by: Vec<String>,
}
```

### Theme

```rust
pub enum Theme {
    Default,       // Dark cyan on slate
    Claude,        // Anthropic terracotta on cream
    Twitter,       // X blue on black
    NeoBrutalism,  // Bold yellow/pink on white
    RetroArcade,   // Neon green on black
    Aurora,        // Purple/teal gradient
    Business,      // Navy on white
}
```

Each theme provides both ANSI 256-color codes (for the TUI) and CSS hex values (for the web UI).

### FlowError

```rust
pub enum FlowError {
    Io(std::io::Error),
    ConfigParse(toml::de::Error),
    Git(String),
    Tmux(String),
    ProjectNotFound(String),
    Database(String),
    NotFound(String),
    BadRequest(String),
    Conflict(String),
    Parse(String),
    Timeout(String),
    CycleDetected(String),
    Json(serde_json::Error),
    Server(String),
}
```

## Usage

```rust
use flow_core::{
    Feature, Task, Config, AgentConfig,
    Theme, ThemeColors, FlowEvent, FlowError, Result,
};

// Load workflow configuration
let config = Config::load()?;
println!("Projects dir: {}", config.projects_dir.display());

// Create agent monitoring config
let agent_config = AgentConfig::new()
    .with_port(3456)
    .with_theme(Theme::Aurora)
    .with_open_browser(true);

// Use theme colors
let theme = Theme::Aurora;
let colors = theme.colors();
println!("Primary CSS color: {}", colors.css_primary);
println!("Primary ANSI: {}", colors.primary.fg());

// Cycle through themes
let next = theme.next(); // Business
```

## Configuration

### Workflow Config (`~/.config/flow/config.toml`)

```toml
projects_dir = "~/Projects"
state_dir = "~/.local/state/flow"
default_base_branch = "main"
```

If the file doesn't exist, sensible defaults are used.

### Agent Config (programmatic)

```rust
let config = AgentConfig::new()     // Defaults
    .with_port(3456)                // Web server port
    .with_theme(Theme::Default)     // Color theme
    .with_max_concurrency(5)        // Max parallel agents
    .with_open_browser(true);       // Auto-open browser
```

## Testing

```bash
cargo test -p flow-core
```

## Related Crates

Every crate in the workspace depends on flow-core:

- **[flow-db](../flow-db/README.md)** - Uses Feature, CreateFeatureInput, FlowError
- **[flow-resolver](../flow-resolver/README.md)** - Uses Feature, FlowError
- **[flow-server](../flow-server/README.md)** - Uses AgentConfig, Task, Theme, FlowEvent
- **[flow-tui](../flow-tui/README.md)** - Uses Theme, ThemeColors
- **[flow-cli](../flow-cli/README.md)** - Uses Config, AgentConfig, Theme

[Back to main README](../../README.md)
