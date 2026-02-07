# flow-cli

The `flow` binary - command-line interface combining git workflow management with AI agent monitoring.

## What it does

`flow-cli` is the main entry point that ties everything together. It's the single binary you install and use for all Flow operations. Think of it as the remote control for your entire development environment.

## Commands

```
flow <command>

Workflow Commands:
  status      Show worktrees, sessions, and project overview
  branch      Create a worktree + tmux session for a branch
  switch      Fuzzy-find and jump to any project
  worktree    Manage git worktrees (list, remove)
  scan        Run security scanners
  sync        Sync state across machines

Agent Monitoring Commands:
  serve       Start web dashboard (localhost:3456)
  monitor     Launch terminal UI (TUI)
  features    Manage features in SQLite database
  theme       List and switch color themes
```

## Usage

### Workflow

```bash
# See everything at a glance
flow status

# Start working on a feature (creates worktree + tmux session)
flow branch feature/auth

# Switch between projects
flow switch
```

### Agent Monitoring

```bash
# Web dashboard
flow serve                    # Opens browser at localhost:3456
flow serve --port 8080        # Custom port
flow serve --no-open          # Don't auto-open browser

# Terminal UI
flow monitor                  # 4 views: Kanban, Agents, Graph, Logs

# Feature management
flow features list            # All features
flow features add "Login"     # Create feature
flow features ready           # Ready to work on
flow features claim 1         # Claim feature #1
flow features pass 1          # Mark as passing
flow features fail 1          # Mark as failing
flow features graph           # Dependency graph

# Themes
flow theme list               # Show 7 available themes
flow theme set aurora         # Switch theme
```

## Architecture

```
flow-cli/
├── main.rs                    - Entry point, clap arg parsing
└── commands/
    ├── mod.rs                 - Command enum and routing
    ├── status.rs              - Status overview
    ├── branch.rs              - Worktree + tmux creation
    ├── switch.rs              - Project switcher
    ├── worktree.rs            - Worktree management
    ├── scan.rs                - Security scanning
    ├── sync_cmd.rs            - State synchronization
    ├── serve.rs               - Web server launcher
    ├── features.rs            - Feature CRUD operations
    ├── monitor.rs             - TUI launcher
    └── theme_cmd.rs           - Theme management
```

## Building

```bash
# Debug build
cargo build -p flow-cli

# Release build (optimized, ~7MB)
cargo build -p flow-cli --release

# Install globally
cargo install --path crates/flow-cli
```

## Related Crates

flow-cli ties together all other crates:

- **[flow-core](../flow-core/README.md)** - Config, types, themes
- **[flow-db](../flow-db/README.md)** - SQLite for `features` commands
- **[flow-resolver](../flow-resolver/README.md)** - Dependency sorting for `features graph/ready`
- **[flow-server](../flow-server/README.md)** - Web server for `serve`
- **[flow-tui](../flow-tui/README.md)** - Terminal UI for `monitor`

[Back to main README](../../README.md)
