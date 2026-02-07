# flow-git

Git worktree operations for the [Agent Flow](https://github.com/aryayt/flow) workspace manager.

## Features

- Branch management (create, list, delete)
- Worktree operations for isolated agent workspaces
- Built on [gitoxide](https://docs.rs/gix) for pure-Rust git operations

## Usage

```toml
[dependencies]
flow-git = "0.1.2"
```

```rust
use flow_git::worktree;
// Create isolated worktrees for parallel agent execution
```

## License

MIT
