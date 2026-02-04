<p align="center">
  <h1 align="center">Flow</h1>
  <p align="center"><strong>Agent -> Flow -> Session -> Branch -> Ship</strong></p>
  <p align="center">Git worktree and tmux workflow manager for parallel development</p>
</p>

<p align="center">
  <a href="https://crates.io/crates/flow-cli"><img src="https://img.shields.io/crates/v/flow-cli.svg" alt="Crates.io"></a>
  <a href="https://github.com/aryayt/flow/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75%2B-orange.svg" alt="Rust"></a>
</p>

<p align="center">
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#commands">Commands</a> •
  <a href="#why-flow">Why Flow?</a> •
  <a href="#architecture">Architecture</a>
</p>

---

```
+----------+    +------+    +---------+    +----------+    +--------+
|  Agent   | -> | Flow | -> | Session | -> | Worktree | -> | Commit |
+----------+    +------+    +---------+    +----------+    +--------+
```

## What is Flow?

Flow eliminates the friction of parallel development. Instead of stashing changes, switching branches, and losing context—create isolated worktrees with dedicated tmux sessions in one command.

```bash
# Before Flow: The painful way
git stash && git checkout -b feature/auth && git stash pop  # Hope nothing conflicts...

# With Flow: One command, full isolation
flow branch feature/auth  # New worktree + tmux session, instantly
```

## Platform Compatibility

| Platform | Status | Notes |
|----------|--------|-------|
| macOS Apple Silicon (M1-M4) | **Fully supported** | Native ARM64 build |
| macOS Intel | **Fully supported** | Native x86_64 build |
| Linux x86_64 | **Fully supported** | Ubuntu, Fedora, Arch, etc. |
| Linux ARM64 (aarch64) | **Fully supported** | Raspberry Pi 4/5, AWS Graviton |
| Windows x86_64 | **WSL2 only** | tmux not available natively |

## Installation

### One-Line Install (Recommended)

```bash
curl -sSf https://raw.githubusercontent.com/aryayt/flow/main/install.sh | bash
```

This detects your OS/architecture, installs prerequisites (if needed), and installs Flow.

<details>
<summary><strong>macOS (Homebrew)</strong></summary>

```bash
# Install prerequisites
brew install git tmux

# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Flow
cargo install flow-cli
```

</details>

<details>
<summary><strong>Ubuntu / Debian</strong></summary>

```bash
# Install prerequisites
sudo apt update
sudo apt install -y git tmux curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Flow
cargo install flow-cli
```

</details>

<details>
<summary><strong>Fedora / RHEL</strong></summary>

```bash
# Install prerequisites
sudo dnf install -y git tmux curl gcc

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Flow
cargo install flow-cli
```

</details>

<details>
<summary><strong>Arch Linux</strong></summary>

```bash
# Install prerequisites
sudo pacman -S --needed git tmux base-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Flow
cargo install flow-cli
```

</details>

<details>
<summary><strong>Raspberry Pi (ARM64)</strong></summary>

```bash
# Ensure 64-bit OS (Raspberry Pi OS 64-bit recommended)
uname -m  # Should show aarch64

# Install prerequisites
sudo apt update
sudo apt install -y git tmux curl build-essential

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Flow (may take a while on Pi)
cargo install flow-cli
```

</details>

<details>
<summary><strong>Windows (WSL2)</strong></summary>

Flow requires tmux, which is not available natively on Windows. Use WSL2:

```powershell
# 1. Install WSL2 (PowerShell as Admin)
wsl --install

# 2. Restart and open Ubuntu
```

Then follow the Ubuntu instructions above inside WSL2.

</details>

### Prerequisites

- **Git** 2.5+ (for worktree support)
- **tmux** (any recent version)
- **Rust** 1.75+ (installed automatically by install script)
- Optional: semgrep, trivy, cargo-audit for security scanning

## Quick Start

```bash
# Show your development dashboard
flow status

# Start working on a feature (creates worktree + tmux session)
flow branch feature/auth

# Fuzzy-find and switch to any project
flow switch

# List all active worktrees
flow worktree list
```

## Why Flow?

### The Problem

Modern development often requires working on multiple features, reviews, and hotfixes simultaneously. Traditional git workflows force you to:

- Stash and unstash changes constantly
- Lose terminal state when switching branches
- Risk merge conflicts from uncommitted work
- Context-switch mentally between different tasks

### The Solution

Flow creates **isolated worktrees** with **dedicated tmux sessions** for each task. Each branch gets its own directory and terminal environment—no stashing, no conflicts, instant context switching.

### Who Benefits?

| Persona | Pain Point | How Flow Helps |
|---------|------------|----------------|
| **Multi-project developers** | Losing context when switching | `flow switch` fuzzy-finds any project instantly |
| **Code reviewers** | PR checkouts pollute main worktree | Each review gets an isolated worktree |
| **AI-assisted developers** | Agent branches conflict with manual work | Parallel worktrees for agents and humans |
| **Open source maintainers** | Managing multiple PRs in flight | Separate worktree per PR, clean state |
| **DevOps engineers** | Hotfixes while mid-feature | Branch off production instantly, no stash |

### Before & After

**Without Flow:**
```bash
# Reviewing a PR while mid-feature
git stash -m "WIP auth feature"
git fetch origin pull/123/head:pr-123
git checkout pr-123
# ... review, test, comment ...
git checkout feature/auth
git stash pop  # Hope it applies cleanly!
```

**With Flow:**
```bash
# Reviewing a PR while mid-feature
flow branch pr-123 --base origin/pull/123/head
# Review in isolated worktree with fresh tmux session
# Your feature work is untouched in its own worktree
```

## Commands

| Command | Description |
|---------|-------------|
| `flow status` | Show dashboard with worktrees, sessions, and projects |
| `flow status --mobile` | Compact status output |
| `flow branch <name>` | Create git worktree and tmux session |
| `flow branch <name> --base <ref>` | Create worktree from specific base |
| `flow switch` | Fuzzy-find and switch to a project |
| `flow switch <query>` | Switch with initial search query |
| `flow worktree list` | List all git worktrees |
| `flow worktree remove <name>` | Remove a worktree |
| `flow scan` | Check available security scanners |
| `flow scan --all` | Run all available scanners |
| `flow sync` | Sync state across machines |

## Configuration

Flow uses `~/.config/flow/config.toml`:

```toml
# Directory to scan for projects
projects_dir = "~/Projects"

# State storage location
state_dir = "~/.local/state/flow"

# Default base branch for new worktrees
default_branch = "main"

# Sync provider (git, s3, or local)
sync_provider = "git"
```

Default values are used if no config file exists.

## Comparison

| Feature | Flow | git worktree (raw) | tmux alone |
|---------|------|-------------------|------------|
| Create worktree + session | One command | Manual setup | N/A |
| Fuzzy project switching | Built-in (skim) | Not included | Not included |
| Session per worktree | Automatic | Manual | Manual |
| Security scanning | Integrated | Not included | Not included |
| Multi-machine sync | Built-in | Not included | Not included |
| Configuration | TOML file | None | .tmux.conf |

## Architecture

Flow is a Rust workspace with focused, single-responsibility crates:

```
                    +-------------+
                    |  flow-cli   |  Binary + Commands
                    +------+------+
                           |
         +-----------------+-----------------+
         |                 |                 |
         v                 v                 v
   +-----------+    +-----------+    +-----------+
   | flow-git  |    |flow-tmux  |    | flow-sync |
   +-----+-----+    +-----+-----+    +-----+-----+
         |                |                 |
         +----------------+-----------------+
                          |
                          v
                    +-----------+
                    | flow-core |  Config, State, Errors
                    +-----------+
```

### File Structure

```
flow/
├── Cargo.toml                 # Workspace definition with shared lints
├── CLAUDE.md                  # AI assistant instructions
├── README.md                  # You are here
├── llms.txt                   # AI/LLM discoverability
│
├── crates/
│   ├── flow-cli/              # Binary entry point
│   │   └── src/
│   │       ├── main.rs        # Clap CLI setup
│   │       ├── ui.rs          # Terminal styling
│   │       └── commands/      # Command handlers
│   │
│   ├── flow-core/             # Shared configuration & state
│   │   └── src/
│   │       ├── config.rs      # ~/.config/flow/config.toml
│   │       ├── state.rs       # ~/.local/state/flow/
│   │       └── project.rs     # Project discovery
│   │
│   ├── flow-git/              # Git worktree operations
│   │   └── src/
│   │       └── worktree.rs    # Worktree CRUD
│   │
│   ├── flow-tmux/             # Tmux integration
│   │   └── src/
│   │       └── session.rs     # Session management
│   │
│   └── flow-sync/             # Multi-machine sync
│       └── src/
│           └── provider.rs    # Trait for sync backends
│
└── extensions/                # TypeScript extensions
    └── src/
        ├── hooks/             # Lifecycle hooks
        └── scanners/          # Security tool wrappers
```

### Rust Best Practices

Flow demonstrates modern Rust patterns:

| Practice | Implementation |
|----------|----------------|
| **Workspace Monorepo** | 5 focused crates with clear boundaries |
| **Shared Lints** | Clippy pedantic + nursery in `[workspace.lints]` |
| **Error Handling** | `thiserror` for libs, `anyhow` for CLI |
| **No Unsafe** | `unsafe_code = "forbid"` workspace-wide |
| **CLI Framework** | `clap` 4.x with derive macros |
| **Config Management** | XDG paths via `dirs` crate, TOML with `serde` |
| **Fuzzy Finding** | Embedded `skim` (not shelling out) |
| **Terminal UI** | `crossterm` + `ratatui` |
| **Pre-commit Hooks** | `lefthook` with fmt, clippy, tests |

## TypeScript Extensions

Located in `extensions/src/`:

- `scanners/` - Wrappers for semgrep, trivy security tools
- `hooks/` - Lifecycle hooks (post-branch, post-switch)

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

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed development guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

<p align="center">
  <sub>Built with Rust. Designed for developers who ship.</sub>
</p>
