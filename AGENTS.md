# Flow - Multi-Agent Collaboration Guide

## For AI Agents Working on This Codebase

### Setup

```bash
git clone https://github.com/aryayt/flow.git
cd flow
cargo build
cargo test  # Must pass before any changes
```

### Task File Format

Write task files to `~/.claude/tasks/{session-id}/{task-id}.json`:

```json
{
  "id": "1",
  "subject": "Task title",
  "description": "What needs to be done",
  "activeForm": "What the agent is currently doing",
  "status": "pending",
  "owner": "claude",
  "blocks": [],
  "blockedBy": []
}
```

Status values: `pending`, `in_progress`, `completed`
Owner values: `claude`, `codex`, `gemini`, `human`

### Agent-Specific Instructions

**Claude Code**: Use native `TaskCreate`/`TaskUpdate` tools. Tasks auto-appear in the viewer.

**Codex CLI**: Write task JSON files directly:
```bash
codex exec --json "Create task file at ~/.claude/tasks/SESSION/ID.json"
```

**Gemini CLI**: Write task JSON files directly:
```bash
gemini -p "Create task file at ~/.claude/tasks/SESSION/ID.json" --output-format json
```

### Git Worktree Isolation

Each agent should work in its own worktree to prevent conflicts:

```bash
flow branch agent/claude-auth    # Creates isolated worktree
flow branch agent/codex-tests    # Another isolated worktree
```

### Feature Management (SQLite)

```bash
flow features add "Feature name" --category backend
flow features list          # See all features
flow features ready         # What's available to work on
flow features claim <id>    # Claim before starting work
flow features pass <id>     # Mark done
flow features fail <id>     # Mark failed
flow features graph         # See dependency tree
```

### Monitoring Agent Work

```bash
flow serve                  # Web dashboard at localhost:3456
flow monitor                # Terminal UI
```

### Crate Responsibilities

| If you need to... | Modify... |
|---|---|
| Add a new type/error | `crates/flow-core/src/` |
| Change database schema | `crates/flow-db/src/migration.rs` |
| Add a feature query | `crates/flow-db/src/feature.rs` |
| Change scheduling logic | `crates/flow-resolver/src/scorer.rs` |
| Add an API endpoint | `crates/flow-server/src/routes/` |
| Add a TUI view | `crates/flow-tui/src/views/` |
| Add a CLI command | `crates/flow-cli/src/commands/` |

### Rules

1. Run `cargo test` before committing
2. Run `cargo clippy --all-targets` and fix warnings
3. Use `dirs` crate for all file paths (cross-platform)
4. Never add `unsafe` code
5. Add tests for new database operations
6. Keep JSON API responses in camelCase
