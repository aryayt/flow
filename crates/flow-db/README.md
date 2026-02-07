# flow-db

SQLite-based persistent storage for feature management with optimized performance and change event logging.

## What it does

Think of `flow-db` as a filing cabinet for your project's features. Instead of keeping feature information in your head or scattered across text files, this crate stores everything in a structured database. It's like having a smart librarian who not only organizes your features but also remembers every change you make to them - who changed what, when, and why.

The database uses SQLite, which is like having a mini-database engine built right into your application. No server setup needed - just a single file on disk.

## Architecture

```
flow-db/
├── lib.rs           - Database handle and connection management
├── feature.rs       - FeatureStore: all feature CRUD operations
├── schema.rs        - Performance optimizations (WAL mode, cache config)
├── migration.rs     - Database schema versioning and upgrades
├── event_log.rs     - Change event tracking and audit log
└── task_sync.rs     - Convert between Feature and Task formats
```

### Key Modules

- **Database** (`lib.rs`): Thread-safe database handle with write lock and read-only connection support
- **FeatureStore** (`feature.rs`): Complete API for creating, reading, updating, and querying features
- **Schema** (`schema.rs`): SQLite performance tuning (WAL mode, 64MB cache, mmap I/O)
- **Migrations** (`migration.rs`): Automatic schema upgrades using version tracking
- **Event Log** (`event_log.rs`): Audit trail for all feature changes
- **Task Sync** (`task_sync.rs`): Bidirectional conversion between Features and Claude Code Tasks

### Database Schema

**Features Table:**
```sql
features (
  id             INTEGER PRIMARY KEY,
  priority       INTEGER,
  category       TEXT,
  name           TEXT,
  description    TEXT,
  steps          TEXT (JSON array),
  passes         INTEGER (boolean),
  in_progress    INTEGER (boolean),
  dependencies   TEXT (JSON array of feature IDs),
  created_at     TEXT (ISO8601),
  updated_at     TEXT (ISO8601)
)
```

**Change Events Table:**
```sql
change_events (
  id            INTEGER PRIMARY KEY,
  feature_id    INTEGER,
  event_type    TEXT,
  field         TEXT,
  old_value     TEXT,
  new_value     TEXT,
  agent         TEXT,
  source        TEXT,
  created_at    TEXT (ISO8601)
)
```

## Usage

### Opening a database

```rust
use flow_db::{Database, FeatureStore};
use flow_core::CreateFeatureInput;

// Open or create a database file
let db = Database::open(Path::new("features.db"))?;

// For testing, use in-memory database
let test_db = Database::open_in_memory()?;
```

### Creating features

```rust
// Get the write connection (behind a mutex)
let conn = db.writer().lock().unwrap();

// Create a single feature
let feature = FeatureStore::create(
    &conn,
    &CreateFeatureInput {
        name: "User Authentication".to_string(),
        description: "Implement JWT-based auth".to_string(),
        priority: Some(100),
        category: "Backend".to_string(),
        steps: vec!["Create user model".to_string()],
        dependencies: vec![],
    },
)?;

println!("Created feature #{} with priority {}", feature.id, feature.priority);
```

### Bulk creation with index-based dependencies

```rust
use flow_core::DependencyRef;

// Create multiple features in one transaction
let inputs = vec![
    CreateFeatureInput {
        name: "Feature A".to_string(),
        dependencies: vec![],
        // ... other fields
    },
    CreateFeatureInput {
        name: "Feature B".to_string(),
        dependencies: vec![DependencyRef::Index { index: 0 }], // Depends on A
        // ... other fields
    },
    CreateFeatureInput {
        name: "Feature C".to_string(),
        dependencies: vec![
            DependencyRef::Index { index: 0 }, // Depends on A
            DependencyRef::Index { index: 1 }, // Depends on B
        ],
        // ... other fields
    },
];

let features = FeatureStore::create_bulk(&conn, &inputs)?;
```

### Querying features

```rust
// Get a specific feature
let feature = FeatureStore::get_by_id(&conn, 42)?.unwrap();

// Get all features (sorted by priority)
let all_features = FeatureStore::get_all(&conn)?;

// Get features ready to work on
let ready = FeatureStore::get_ready(&conn)?;

// Get blocked features
let blocked = FeatureStore::get_blocked(&conn)?;

// Get statistics
let stats = FeatureStore::get_stats(&conn)?;
println!("Total: {}, Passing: {}, Failing: {}, In Progress: {}, Blocked: {}",
    stats.total, stats.passing, stats.failing, stats.in_progress, stats.blocked);
```

### Atomic feature claiming

```rust
// Atomically claim a feature for work (prevents race conditions)
match FeatureStore::claim_and_get(&conn, feature_id) {
    Ok(feature) => {
        println!("Claimed feature: {}", feature.name);
        // Do work...
        FeatureStore::mark_passing(&conn, feature_id)?;
    }
    Err(FlowError::Conflict(_)) => {
        println!("Feature already claimed by another agent");
    }
    Err(e) => return Err(e),
}
```

### Managing dependencies

```rust
// Add a dependency
FeatureStore::add_dependency(&conn, feature_id, dep_id)?;

// Remove a dependency
FeatureStore::remove_dependency(&conn, feature_id, dep_id)?;

// Set all dependencies at once (replaces existing)
FeatureStore::set_dependencies(&conn, feature_id, &[dep1, dep2, dep3])?;

// Get dependency graph with dependents computed
let graph = FeatureStore::get_graph(&conn)?;
for node in graph {
    println!("{} depends on {:?}, blocks {:?}",
        node.name, node.dependencies, node.dependents);
}
```

### Logging changes

```rust
use flow_db::event_log;

// Log a change event
event_log::log_event(
    &conn,
    feature_id,
    "status_change",
    Some("status"),
    Some("pending"),
    Some("in_progress"),
    Some("agent-1"),
    "api",
)?;

// Get all events for a feature
let events = event_log::get_events(&conn, feature_id)?;
for event in events {
    println!("{}: {} changed {} from {:?} to {:?}",
        event.created_at, event.agent.unwrap_or("unknown"),
        event.field.unwrap(), event.old_value, event.new_value);
}
```

## API Reference

| Function | Description |
|----------|-------------|
| `Database::open(path)` | Open database file with automatic migrations |
| `Database::open_in_memory()` | Create in-memory database for testing |
| `FeatureStore::create(conn, input)` | Create a single feature |
| `FeatureStore::create_bulk(conn, inputs)` | Create multiple features atomically |
| `FeatureStore::get_by_id(conn, id)` | Get feature by ID |
| `FeatureStore::get_all(conn)` | Get all features sorted by priority |
| `FeatureStore::get_ready(conn)` | Get features ready to work on |
| `FeatureStore::get_blocked(conn)` | Get features blocked by dependencies |
| `FeatureStore::get_stats(conn)` | Get aggregate statistics |
| `FeatureStore::get_graph(conn)` | Get full dependency graph |
| `FeatureStore::claim_and_get(conn, id)` | Atomically claim a feature |
| `FeatureStore::mark_passing(conn, id)` | Mark feature as passing |
| `FeatureStore::mark_failing(conn, id)` | Mark feature as failing |
| `FeatureStore::mark_in_progress(conn, id)` | Mark feature in progress |
| `FeatureStore::clear_in_progress(conn, id)` | Clear in-progress flag |
| `FeatureStore::skip(conn, id)` | Move feature to end of priority queue |
| `FeatureStore::add_dependency(conn, id, dep)` | Add a dependency |
| `FeatureStore::remove_dependency(conn, id, dep)` | Remove a dependency |
| `FeatureStore::set_dependencies(conn, id, deps)` | Set all dependencies |
| `event_log::log_event(...)` | Log a change event |
| `event_log::get_events(conn, feature_id)` | Get all events for a feature |

## Performance Optimizations

The database is tuned for high-performance operations:

- **WAL Mode**: Write-Ahead Logging for better concurrency
- **64MB Cache**: Large in-memory cache for hot data
- **Memory-Mapped I/O**: Direct file mapping for faster reads
- **Busy Timeout**: 1000ms retry on lock contention
- **Indexes**: Optimized indexes on status, priority, and foreign keys

Typical query performance:
- Feature lookup by ID: ~0.1ms
- Get all features: ~1-2ms for 100 features
- Claim operation: ~0.5ms (atomic)

## Testing

```bash
# Run all tests
cargo test -p flow-db

# Run with output
cargo test -p flow-db -- --nocapture

# Run specific test
cargo test -p flow-db test_bulk_create_with_index_dependencies
```

## Related Crates

- **[flow-core](../flow-core/README.md)**: Core types (Feature, Task, CreateFeatureInput)
- **[flow-resolver](../flow-resolver/README.md)**: Dependency resolution and topological sorting
- **[flow-server](../flow-server/README.md)**: Web server that uses this database
- **[flow-mcp](../flow-mcp/README.md)**: MCP server exposing database operations as tools

[Back to main README](../../README.md)
