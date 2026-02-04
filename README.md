# Flow

A Rust CLI for multi-agent development workflows, managing git worktrees, tmux sessions, and project switching.

## Features

- **Git Worktree Management** - Create, list, and remove worktrees with automatic branch setup
- **TMUX Integration** - Automatically create tmux sessions for each worktree
- **Fuzzy Project Switching** - Instantly switch between projects with embedded skim fuzzy finder
- **Security Scanning** - Run semgrep, trivy, and cargo-audit scans from one command
- **Beautiful Dashboard** - Terminal status display with worktrees, sessions, and project info
- **TypeScript Extensions** - Extensible hooks and scanner wrappers

## Installation

### From Source

```bash
git clone https://github.com/aryayt/flow.git
cd flow
cargo install --path crates/flow-cli
```

### Prerequisites

- Rust 1.75.0+
- Git
- tmux
- Optional: semgrep, trivy, cargo-audit for security scanning

## Quick Start

```bash
# Show status dashboard
flow status

# Create a new worktree with tmux session
flow branch feature/my-feature

# List all worktrees
flow worktree list

# Fuzzy-find and switch projects
flow switch

# Run security scans
flow scan --all
```

## Commands

| Command | Description |
|---------|-------------|
| `flow status` | Show status dashboard with worktrees, sessions, and projects |
| `flow status --mobile` | Compact status output |
| `flow branch <name>` | Create git worktree and tmux session |
| `flow branch <name> --base <branch>` | Create worktree from specific base branch |
| `flow switch` | Fuzzy-find and switch to a project |
| `flow switch <query>` | Switch with initial search query |
| `flow worktree list` | List all git worktrees |
| `flow worktree remove <name>` | Remove a worktree |
| `flow scan` | Check available security scanners |
| `flow scan --all` | Run all available scanners |
| `flow sync` | Sync state across machines |

## Configuration

Flow uses a TOML configuration file at `~/.config/flow/config.toml`:

```toml
# Projects directory to scan
projects_dir = "~/Projects"

# State storage directory
state_dir = "~/.local/state/flow"

# Default base branch for new worktrees
default_branch = "main"

# Sync provider (git, s3, or local)
sync_provider = "git"
```

Default values are used if no config file exists.

## Architecture

```
crates/
├── flow-cli/      # Binary entry point (clap CLI)
├── flow-core/     # Config, state, and project discovery
├── flow-git/      # Git worktree operations
├── flow-tmux/     # TMUX session/window management
└── flow-sync/     # Multi-machine state sync
```

### TypeScript Extensions

Located in `extensions/src/`:

- `scanners/` - Wrappers for semgrep, trivy security tools
- `hooks/` - Lifecycle hooks (post-branch, post-switch)

Build with:

```bash
cd extensions && npm install && npm run build
```

## Development

```bash
# Build
cargo build

# Run tests
cargo nextest run

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt

# Watch mode
bacon
```

## License

MIT License - see [LICENSE](LICENSE) for details.
