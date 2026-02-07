# Flow - Agent Instructions

## What This Project Is

Flow is a Rust CLI that combines git worktree management with AI agent task monitoring. Binary name: `flow`. Published on crates.io as `flow-cli`.

## Quick Commands

```bash
# Build
cargo build                    # Debug build
cargo build --release          # Release build (~7MB binary)

# Test (56 tests across 3 crates)
cargo test                     # Run all tests
cargo test -p flow-db          # 22 tests: SQLite CRUD, transactions
cargo test -p flow-resolver    # 32 tests: topo sort, cycles, scoring
cargo test -p flow-server      # 2 tests: helpers

# Lint
cargo fmt --all -- --check     # Check formatting
cargo clippy --all-targets     # Workspace uses pedantic + nursery lints

# Run
cargo run -p flow-cli -- status       # Show worktrees/sessions
cargo run -p flow-cli -- serve        # Web dashboard on :3456
cargo run -p flow-cli -- monitor      # Terminal UI
cargo run -p flow-cli -- features list # List features in SQLite
cargo run -p flow-cli -- theme list   # Show 7 themes
```

## Architecture

11-crate Cargo workspace. Dependencies flow downward:

```
flow-cli (binary)
  ├── flow-server (axum web server, SSE, WebSocket)
  ├── flow-tui (ratatui terminal UI)
  ├── flow-git (worktree ops via gix)
  ├── flow-tmux (session management)
  └── flow-sync (state sync)
        ├── flow-db (SQLite, rusqlite with WAL)
        ├── flow-resolver (Kahn's topo sort, DFS cycle detection)
        └── flow-core (types, config, themes, errors)
```

Stubs (not yet implemented): flow-mcp, flow-orchestrator

## Key Conventions

- **Error handling**: `thiserror` in libraries, `anyhow` in CLI
- **Serialization**: `serde` with `#[serde(rename_all = "camelCase")]` for JSON
- **Database**: SQLite with WAL mode, `rusqlite` bundled feature
- **Async**: `tokio` for server, sync code for CLI and DB operations
- **Paths**: Always use `dirs` crate (`home_dir()`, `config_dir()`), never hardcode
- **Lints**: `unsafe_code = "forbid"`, clippy pedantic + nursery workspace-wide
- **Tests**: Standard `#[test]`, in-memory SQLite for DB tests
- **Themes**: 7 themes defined once in flow-core, projected to ANSI (TUI) and CSS (web)

## File Paths

- Task JSON files: `~/.claude/tasks/{session-id}/{task-id}.json`
- Project metadata: `~/.claude/projects/`
- Workflow config: `~/.config/flow/config.toml`
- Agent data: `~/.claude/data/`

Use `dirs::home_dir()` for `~`, `dirs::config_dir()` for config, `Path::join()` for construction. Never string-concatenate paths.

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

## Publishing

Published to crates.io as 9 crates (flow-mcp and flow-orchestrator are `publish = false`).

Publishing order (dependencies first):
1. flow-core
2. flow-db, flow-resolver, flow-git, flow-tmux, flow-sync
3. flow-server, flow-tui
4. flow-cli

## Do NOT

- Modify the task JSON file format without updating both server and frontend
- Remove SSE event broadcasting (core real-time mechanism)
- Use `std::env::var("HOME")` instead of `dirs::home_dir()`
- Use string concatenation for file paths
- Add `unsafe` code (forbidden by workspace lint)
- Skip `cargo test` before committing
