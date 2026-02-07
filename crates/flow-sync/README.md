# flow-sync

Multi-machine state synchronization for the [Agent Flow](https://github.com/aryayt/flow) workspace manager.

## Features

- Pluggable sync providers (git-based, file-based)
- Workspace state replication across machines
- Conflict resolution for concurrent edits

## Usage

```toml
[dependencies]
flow-sync = "0.1.2"
```

> **Note:** This crate is currently a work-in-progress. The sync provider API is subject to change.

## License

MIT
