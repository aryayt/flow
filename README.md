<p align="center">
  <h1 align="center">Flow</h1>
  <p align="center"><strong>Agent -> Flow -> Session -> Branch -> Ship</strong></p>
  <p align="center">Git workflow manager + AI agent monitoring for parallel development</p>
</p>

<p align="center">
  <a href="https://crates.io/crates/flow-cli"><img src="https://img.shields.io/crates/v/flow-cli.svg" alt="Crates.io"></a>
  <a href="https://github.com/aryayt/flow/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.80%2B-orange.svg" alt="Rust"></a>
  <a href="https://github.com/aryayt/flow/actions"><img src="https://github.com/aryayt/flow/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> &bull;
  <a href="#commands">Commands</a> &bull;
  <a href="#agent-monitoring">Agent Monitoring</a> &bull;
  <a href="#architecture">Architecture</a> &bull;
  <a href="#crate-documentation">Crate Docs</a> &bull;
  <a href="#development">Development</a>
</p>

---

## What is Flow?

Flow is two things in one CLI:

1. **Workflow Manager** - Create isolated git worktrees with tmux sessions in one command
2. **Agent Monitor** - Track AI agent tasks (Claude, Codex, Gemini) with a real-time Kanban board, TUI, and web dashboard

```
+----------+    +------+    +---------+    +----------+    +--------+
|  Agent   | -> | Flow | -> | Session | -> | Worktree | -> | Commit |
+----------+    +------+    +---------+    +----------+    +--------+
     |                          |
     v                          v
 +--------+              +----------+
 | SQLite |              | Kanban   |
 | Features|             | Board    |
 +--------+              +----------+
```

```bash
# Git workflow: one command, full isolation
flow branch feature/auth    # New worktree + tmux session

# Agent monitoring: real-time Kanban board
flow serve                  # Web dashboard at localhost:3456
flow monitor                # Terminal UI with 4 views
flow features list          # SQLite-backed feature tracking
```

---

## Quick Start

### Installation

```bash
# From source (recommended for development)
git clone https://github.com/aryayt/flow.git
cd flow
cargo build --release
# Binary is at target/release/flow

# From crates.io (when published)
cargo install flow-cli
```

### Prerequisites

- **Rust** 1.80+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- **Git** 2.5+ (for worktree support)
- **tmux** (for session management)

### Your First Commands

```bash
# 1. See your development dashboard
flow status

# 2. Create an isolated worktree for a feature
flow branch feature/auth

# 3. Start the agent monitoring web dashboard
flow serve

# 4. Or use the terminal UI
flow monitor

# 5. Manage features in SQLite
flow features add "Login page" --category "frontend"
flow features list
flow features ready
```

---

## Commands

### Workflow Commands

These help you manage parallel development with git worktrees and tmux:

| Command | What it Does |
|---------|-------------|
| `flow status` | Shows your worktrees, tmux sessions, and projects |
| `flow status --mobile` | Compact output for small screens |
| `flow branch <name>` | Creates a worktree + tmux session for the branch |
| `flow branch <name> --base <ref>` | Creates from a specific base (e.g., a PR) |
| `flow switch` | Fuzzy-find and jump to any project |
| `flow worktree list` | Lists all git worktrees |
| `flow worktree remove <name>` | Cleans up a worktree |
| `flow scan --all` | Runs security scanners (semgrep, trivy) |
| `flow sync` | Syncs state across machines |

### Agent Monitoring Commands

These help you track AI agent tasks and manage features:

| Command | What it Does |
|---------|-------------|
| `flow serve` | Starts web dashboard at `localhost:3456` |
| `flow serve --port 8080` | Custom port |
| `flow serve --no-open` | Don't auto-open browser |
| `flow monitor` | Launches the terminal UI (TUI) |
| `flow features list` | Lists all features in the database |
| `flow features add "name"` | Creates a new feature |
| `flow features ready` | Shows features ready to work on |
| `flow features claim <id>` | Claims a feature for work |
| `flow features pass <id>` | Marks a feature as passing |
| `flow features fail <id>` | Marks a feature as failing |
| `flow features graph` | Shows dependency graph |
| `flow theme list` | Shows 7 available themes |
| `flow theme set aurora` | Switches to Aurora theme |

### Available Themes

| Theme | Style |
|-------|-------|
| **Default** | Dark cyan on slate |
| **Claude** | Anthropic terracotta on cream |
| **Twitter** | X blue on black |
| **Neo Brutalism** | Bold yellow/pink on white |
| **Retro Arcade** | Neon green on black |
| **Aurora** | Purple/teal gradient |
| **Business** | Navy on white, minimal |

---

## Agent Monitoring

### How it Works

Flow monitors AI agent tasks in real-time. Agents (Claude Code, Codex CLI, Gemini CLI) write task files to `~/.claude/tasks/`. Flow watches these files and displays them on a Kanban board.

```
~/.claude/tasks/
  session-abc/
    1.json  ->  { id: "1", subject: "Fix login", status: "in_progress", owner: "claude" }
    2.json  ->  { id: "2", subject: "Add tests", status: "pending", blockedBy: ["1"] }
```

### Three Ways to View

**1. Web Dashboard** (`flow serve`)
- Opens at `http://localhost:3456`
- Real-time updates via Server-Sent Events (SSE) and WebSocket
- Drag-and-drop Kanban board
- Works in any browser

**2. Terminal UI** (`flow monitor`)
- 4 views: Kanban, Agents, Graph, Logs
- Responsive: adapts to terminal size (Full/Compact/Mobile)
- Keyboard-driven: `1-4` switch views, `j/k` navigate, `t` cycle themes

**3. CLI** (`flow features`)
- SQLite-backed feature database
- Dependency tracking with cycle detection
- Priority-aware scheduling (lower number = higher priority)

### Feature Database

The `flow features` commands use a SQLite database with:
- **Atomic state transitions** - Features move through: pending -> in_progress -> passing/failing
- **Dependency resolution** - Topological sort ensures features run in the right order
- **Cycle detection** - Catches circular dependencies before they cause problems
- **Scheduling scores** - Prioritizes features that unblock the most work

```bash
# Create features with dependencies
flow features add "Database schema" --category backend
flow features add "API endpoints" --category backend
flow features add "Frontend components" --category frontend

# View in dependency order
flow features graph
```

---

## Architecture

Flow is a Rust workspace with 11 focused crates. Each crate does one thing well.

```
                         +-------------+
                         |  flow-cli   |  Binary ("flow")
                         +------+------+  All commands live here
                                |
          +---------------------+---------------------+
          |           |         |         |            |
          v           v         v         v            v
    +-----------+ +--------+ +--------+ +------+ +-----------+
    | flow-git  | | flow-  | | flow-  | | flow-| | flow-     |
    | Worktrees | | tmux   | | sync   | | tui  | | server    |
    +-----------+ +--------+ +--------+ +------+ +-----------+
          |           |         |         |  |         |
          +-----+-----+---------+---------+  |    +----+----+
                |                             |    |         |
                v                             |    v         v
          +-----------+                       |  +------+ +-----+
          | flow-core |  Types, Config,       |  | flow-| |flow-|
          | Errors, Themes, Events            |  | mcp  | |orch.|
          +-----------+                       |  +------+ +-----+
                ^                             |    |
                |                             v    v
          +-----+------+              +------------+
          | flow-       |             | flow-db    |
          | resolver    |             | SQLite     |
          | Topo sort,  |<----------->| Features   |
          | Scheduling  |             | Events     |
          +-------------+             +------------+
```

### Crate Overview

| Crate | Purpose | Lines | Tests | README |
|-------|---------|-------|-------|--------|
| [flow-core](crates/flow-core/) | Shared types: Feature, Task, Theme, Config, Errors | ~550 | - | [README](crates/flow-core/README.md) |
| [flow-db](crates/flow-db/) | SQLite database with WAL mode, feature CRUD, event logging | ~1,750 | 22 | [Docs](crates/flow-db/README.md) |
| [flow-resolver](crates/flow-resolver/) | Topological sort, cycle detection, scheduling scores | ~880 | 32 | [Docs](crates/flow-resolver/README.md) |
| [flow-server](crates/flow-server/) | axum web server with SSE + WebSocket, file watching | ~1,280 | 2 | [Docs](crates/flow-server/README.md) |
| [flow-tui](crates/flow-tui/) | ratatui terminal UI with 4 views and 3 layout modes | ~940 | - | [Docs](crates/flow-tui/README.md) |
| [flow-cli](crates/flow-cli/) | clap CLI binary with 10 commands | ~520 | - | [README](crates/flow-cli/README.md) |
| [flow-git](crates/flow-git/) | Git worktree operations via gitoxide | ~200 | - | - |
| [flow-tmux](crates/flow-tmux/) | tmux session management | ~100 | - | - |
| [flow-sync](crates/flow-sync/) | Multi-machine state sync | ~80 | - | - |
| [flow-mcp](crates/flow-mcp/) | MCP server for Claude Code (planned) | stub | - | [Docs](crates/flow-mcp/README.md) |
| [flow-orchestrator](crates/flow-orchestrator/) | Agent process management (planned) | stub | - | [Docs](crates/flow-orchestrator/README.md) |

### How Data Flows

```
1. Agent writes task JSON     ->  ~/.claude/tasks/{session}/{id}.json
2. File watcher detects it    ->  notify crate with debouncing
3. SSE/WS broadcast           ->  All connected clients get the update
4. Web UI / TUI updates       ->  Kanban board shows new task
```

```
1. User creates feature       ->  flow features add "Login page"
2. SQLite stores it           ->  WAL mode for concurrent access
3. Resolver sorts deps        ->  Kahn's algorithm topological sort
4. Scorer ranks by priority   ->  unblock_count * 1000 + depth_score * 100 + priority * 10
5. Agent claims next ready    ->  Atomic UPDATE with WHERE guard
```

### File Structure

```
flow/
├── Cargo.toml                 # Workspace: 11 crates, shared lints
├── README.md                  # You are here
├── .github/workflows/ci.yml   # CI: test, clippy, format
│
├── crates/
│   ├── flow-cli/              # Binary ("flow") - all commands
│   │   └── src/commands/      # branch, switch, serve, features, monitor, theme...
│   │
│   ├── flow-core/             # Shared types and configuration
│   │   └── src/
│   │       ├── config.rs      # Workflow config (~/.config/flow/config.toml)
│   │       ├── agent.rs       # AgentConfig (port, themes, concurrency)
│   │       ├── feature.rs     # Feature, FeatureStats, FeatureGraphNode
│   │       ├── task.rs        # Task, SessionListItem, SessionMeta
│   │       ├── theme.rs       # 7 themes with ANSI + CSS colors
│   │       ├── event.rs       # FlowEvent (TaskUpdate, FeatureUpdate, etc.)
│   │       └── error.rs       # FlowError (12 variants)
│   │
│   ├── flow-db/               # SQLite feature management
│   │   └── src/
│   │       ├── feature.rs     # CRUD + atomic claim/pass/fail
│   │       ├── schema.rs      # Tables + WAL + performance pragmas
│   │       ├── migration.rs   # Schema versioning
│   │       ├── event_log.rs   # Change audit trail
│   │       └── task_sync.rs   # JSON <-> SQLite bidirectional sync
│   │
│   ├── flow-resolver/         # Dependency resolution
│   │   └── src/
│   │       ├── topo.rs        # Kahn's algorithm with priority heap
│   │       ├── cycle.rs       # DFS cycle detection (depth limit: 50)
│   │       └── scorer.rs      # Scheduling score computation
│   │
│   ├── flow-server/           # Web server (axum)
│   │   └── src/
│   │       ├── routes/        # REST API: sessions, tasks, features, theme
│   │       ├── sse.rs         # Server-Sent Events
│   │       ├── ws.rs          # WebSocket handler
│   │       ├── watcher.rs     # File system watcher (notify crate)
│   │       └── state.rs       # Shared application state
│   │
│   ├── flow-tui/              # Terminal UI (ratatui)
│   │   └── src/
│   │       ├── app.rs         # App state, key handling, rendering
│   │       ├── views/         # kanban, agents, graph, logs, help
│   │       └── theme.rs       # ANSI color mapping from flow-core themes
│   │
│   ├── flow-git/              # Git worktree operations
│   ├── flow-tmux/             # tmux session management
│   ├── flow-sync/             # Multi-machine sync
│   ├── flow-mcp/              # MCP server (planned)
│   └── flow-orchestrator/     # Agent orchestrator (planned)
│
└── extensions/                # TypeScript security scanners
```

---

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/aryayt/flow.git
cd flow

# Build in debug mode (fast compile, slow runtime)
cargo build

# Build in release mode (slow compile, fast runtime)
cargo build --release

# The binary is at:
# Debug:   target/debug/flow
# Release: target/release/flow (7MB, stripped)
```

### Running Tests

```bash
# Run ALL tests (56 tests across 3 crates)
cargo test

# Run tests for a specific crate
cargo test -p flow-db         # 22 tests: CRUD, transactions, event logging
cargo test -p flow-resolver   # 32 tests: topo sort, cycles, scoring
cargo test -p flow-server     # 2 tests: server state, metadata

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test -p flow-db test_create_and_get
```

### Linting and Formatting

```bash
# Check formatting
cargo fmt --all -- --check

# Fix formatting
cargo fmt --all

# Run clippy (workspace uses pedantic + nursery lints)
cargo clippy --all-targets

# Watch mode (requires bacon: cargo install bacon)
bacon
```

### Project Conventions

| Convention | Details |
|-----------|---------|
| **Rust version** | 1.80+ (set in `Cargo.toml` `rust-version`) |
| **Lints** | Clippy pedantic + nursery, `unsafe_code = "forbid"` |
| **Error handling** | `thiserror` in libraries, `anyhow` in CLI |
| **Serialization** | `serde` with `camelCase` rename for JSON APIs |
| **Database** | SQLite with WAL mode, `rusqlite` with bundled feature |
| **Async** | `tokio` for server, sync code for CLI/DB |
| **Testing** | Standard `#[test]`, in-memory SQLite for DB tests |

---

## Crate Documentation

Each crate has its own README with detailed documentation:

- **[flow-core](crates/flow-core/README.md)** - Shared types, configuration, themes, errors, events
- **[flow-cli](crates/flow-cli/README.md)** - CLI binary with 10 commands
- **[flow-db](crates/flow-db/README.md)** - How the SQLite database works, schema design, transaction patterns
- **[flow-resolver](crates/flow-resolver/README.md)** - Topological sort algorithm, cycle detection, scheduling scores
- **[flow-server](crates/flow-server/README.md)** - REST API endpoints, SSE/WebSocket, file watching
- **[flow-tui](crates/flow-tui/README.md)** - Terminal UI views, keyboard shortcuts, responsive layouts
- **[flow-mcp](crates/flow-mcp/README.md)** - MCP server for Claude Code integration (planned)
- **[flow-orchestrator](crates/flow-orchestrator/README.md)** - Agent process management (planned)

---

## Why Flow?

### The Problem

Modern development requires juggling multiple tasks simultaneously. Traditional git workflows force you to stash and unstash, lose terminal state, and risk merge conflicts. When AI agents join the picture, the complexity multiplies.

### The Solution

Flow gives you **isolated worktrees** for parallel development and **real-time monitoring** for AI agents. Each branch gets its own directory and terminal. Agent tasks appear on a Kanban board automatically.

### Who Benefits?

| You are a... | Your pain point | Flow helps by... |
|--------------|----------------|-----------------|
| **Developer** | Losing context when switching branches | `flow branch` creates isolated worktrees |
| **AI-assisted dev** | Can't see what agents are doing | `flow serve` shows a real-time Kanban board |
| **Team lead** | Tracking features across agents | `flow features` with dependency-aware scheduling |
| **Code reviewer** | PR checkouts mess up your work | Each review gets its own worktree |
| **DevOps engineer** | Hotfixes while mid-feature | Branch off production instantly |

---

## Configuration

### Workflow Config

`~/.config/flow/config.toml`:

```toml
projects_dir = "~/Projects"
state_dir = "~/.local/state/flow"
default_branch = "main"
sync_provider = "git"
```

### Agent Monitoring Config

The agent monitoring system uses sensible defaults:

| Setting | Default | Override |
|---------|---------|---------|
| Web server port | 3456 | `flow serve --port 8080` |
| Tasks directory | `~/.claude/tasks/` | Set via `AgentConfig` |
| Database location | `~/.claude/data/default.db` | `flow features --db /path/to/db` |
| Open browser | Yes | `flow serve --no-open` |
| Theme | Default (dark cyan) | `flow theme set aurora` |

---

## Platform Compatibility

| Platform | Status | Notes |
|----------|--------|-------|
| macOS Apple Silicon (M1-M4) | Fully supported | Native ARM64 |
| macOS Intel | Fully supported | Native x86_64 |
| Linux x86_64 | Fully supported | Ubuntu, Fedora, Arch |
| Linux ARM64 | Fully supported | Raspberry Pi 4/5 |
| Windows | WSL2 only | tmux not available natively |

---

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust. 11 crates. 56 tests. 7 themes. 0 unsafe code.</sub>
</p>
