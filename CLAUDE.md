# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
# Rust
cargo build                                              # Build
cargo check --all-targets --all-features                 # Quick check
cargo clippy --all-targets --all-features -- -D warnings # Lint
cargo fmt                                                # Format
cargo nextest run                                        # Run all tests
cargo nextest run -p flow-git                            # Test single crate
cargo nextest run test_worktree_create                   # Run specific test
bacon                                                    # Watch mode (t=test, c=clippy)

# TypeScript extensions
cd extensions && npm run check                           # Lint + type check
cd extensions && npm run build                           # Compile
```

## Architecture

**flow** is a Rust CLI for multi-agent development workflows, managing git worktrees, TMUX sessions, and project switching.

### Workspace Crates

```
crates/
├── flow-cli/      # Binary entry point (clap CLI) - orchestrates other crates
├── flow-core/     # Config (~/.config/flow/config.toml), state, project discovery
├── flow-git/      # Git worktree operations (using shell commands, gix available)
├── flow-tmux/     # TMUX session/window management (using shell commands)
└── flow-sync/     # Multi-machine state sync (stub - uses provider trait)
```

**Dependency flow**: `flow-cli` → `flow-{core,git,tmux,sync}` → `flow-core`

### Key Patterns

- **Error handling**: `thiserror` for library crates, `anyhow` for CLI
- **Workspace lints**: Defined in root `Cargo.toml` - clippy pedantic/nursery enabled, unsafe forbidden
- **No async in commands**: Functions are sync; remove `async` if no `.await` (clippy enforces)
- **Config paths**: `~/.config/flow/config.toml` for config, `~/.local/state/flow/` for state

### CLI Commands

| Command | Handler | Description |
|---------|---------|-------------|
| `flow branch <name>` | `commands/branch.rs` | Create worktree + tmux session |
| `flow switch [query]` | `commands/switch.rs` | Fuzzy-find projects with skim |
| `flow worktree list\|remove` | `commands/worktree.rs` | Manage worktrees |
| `flow sync` | `commands/sync.rs` | Sync state across machines |
| `flow scan [--all]` | `commands/scan.rs` | Run security scanners |
| `flow status [--mobile]` | `commands/status.rs` | Show dashboard |

### TypeScript Extensions

Located in `extensions/src/`:
- `scanners/` - Wrappers for semgrep, trivy
- `hooks/` - Lifecycle hooks (post-branch, post-switch)

Uses Biome for linting/formatting. Build with `npm run build`.

## Adding New Features

### New CLI Command
1. Create `crates/flow-cli/src/commands/newcmd.rs`
2. Add to `commands/mod.rs` exports
3. Add variant to `Commands` enum in `main.rs`
4. Implement sync handler (not async unless needed)

### New Crate
1. Create `crates/flow-newcrate/` with `Cargo.toml` using workspace inheritance
2. Add to `members` in root `Cargo.toml`
3. Use `[lints] workspace = true` in crate's Cargo.toml

## Pre-Commit Hooks

Lefthook runs on commit:
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo nextest run --no-fail-fast`

## Key Dependencies

- **clap** 4.5 - CLI parsing with derive
- **gix** 0.66 - Git operations (currently using shell commands instead)
- **skim** 0.10 - Fuzzy finder (embedded, not CLI)
- **ratatui** 0.26 - TUI (for future status dashboard)
- **tmux_interface** 0.3 - TMUX control (currently using shell commands)
