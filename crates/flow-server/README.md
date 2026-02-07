# flow-server

High-performance web server with real-time updates via Server-Sent Events (SSE) and WebSockets, built on axum and tokio.

## What it does

`flow-server` is the web backend for the Flow Kanban board. It's like a smart waiter in a restaurant:
- **Serves files**: Delivers the web interface (HTML, CSS, JavaScript)
- **Provides data**: Responds to API requests for tasks, sessions, and features
- **Watches for changes**: Monitors the file system and instantly notifies all connected browsers
- **Manages real-time updates**: Uses SSE and WebSockets to push changes to clients without them asking

It's built for **speed** and **concurrency**: sub-millisecond response times, handles hundreds of concurrent connections, and uses async I/O throughout.

## Architecture

```
flow-server/
├── lib.rs              - Server setup and router configuration
├── state.rs            - Shared application state and metadata cache
├── routes/
│   ├── mod.rs          - Route organization
│   ├── sessions.rs     - List/get sessions
│   ├── tasks.rs        - Task CRUD operations
│   ├── features.rs     - Feature API (if database enabled)
│   └── theme.rs        - Theme management
├── sse.rs              - Server-Sent Events handler
├── ws.rs               - WebSocket handler
├── watcher.rs          - File system watcher (notify crate)
├── error.rs            - Error types and conversions
└── helpers.rs          - Utility functions
```

### Data Flow

```
File System Change
       ↓
  File Watcher (notify)
       ↓
  Broadcast Channel (tokio)
       ↓         ↓
      SSE       WebSocket
       ↓         ↓
   Browser    Agent CLI
```

## Usage

### Starting the server

```rust
use flow_server::run_server;
use flow_core::AgentConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure server
    let config = AgentConfig {
        port: 3456,
        tasks_dir: Some("/Users/username/.claude/tasks".into()),
        projects_dir: Some("/Users/username/.claude/projects".into()),
        public_dir: Some("./public".into()),
        open_browser: true,
        ..Default::default()
    };

    // Run server (blocks until shutdown)
    run_server(config).await?;
    Ok(())
}
```

### Building a custom router

```rust
use flow_server::{build_router, AppState, MetadataCache};
use tokio::sync::{broadcast, RwLock};
use std::sync::Arc;

// Create shared state
let (tx, _rx) = broadcast::channel(256);
let state = Arc::new(AppState {
    tasks_dir: "/tmp/tasks".into(),
    projects_dir: "/tmp/projects".into(),
    tx,
    metadata_cache: RwLock::new(MetadataCache::new()),
    db: None, // Optional database
});

// Build router
let app = build_router(state);

// Start server
let listener = tokio::net::TcpListener::bind("127.0.0.1:3456").await?;
axum::serve(listener, app).await?;
```

### Client-side: Consuming SSE events

```javascript
// Connect to Server-Sent Events
const eventSource = new EventSource('/api/events');

eventSource.onmessage = (event) => {
  const data = JSON.parse(event.data);

  switch (data.type) {
    case 'connected':
      console.log('Connected to server');
      break;
    case 'update':
      console.log('File changed:', data.sessionId, data.file);
      refreshTaskList();
      break;
    case 'metadata-update':
      console.log('Project metadata changed');
      refreshSessions();
      break;
  }
};

eventSource.onerror = (error) => {
  console.error('SSE error:', error);
};
```

### Client-side: Using WebSocket

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:3456/api/ws');

ws.onopen = () => {
  console.log('WebSocket connected');

  // Send agent status update
  ws.send(JSON.stringify({
    type: 'agent-status',
    agent: 'codex',
    status: 'working',
    task: 'Implementing OAuth'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};

// Send periodic ping
setInterval(() => {
  ws.send(JSON.stringify({ type: 'ping' }));
}, 30000);
```

### Making API requests

```rust
use reqwest;
use serde_json::Value;

// Get all sessions
let sessions: Value = reqwest::get("http://localhost:3456/api/sessions")
    .await?
    .json()
    .await?;

println!("Sessions: {}", sessions);

// Get tasks for a specific session
let tasks: Value = reqwest::get("http://localhost:3456/api/tasks/all")
    .await?
    .json()
    .await?;

println!("Tasks: {}", tasks);

// Delete a task
reqwest::Client::new()
    .delete("http://localhost:3456/api/tasks/session-123/task-456")
    .send()
    .await?;
```

## API Reference

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/api/sessions` | List all available sessions |
| `GET` | `/api/sessions/:session_id` | Get details for a specific session |
| `GET` | `/api/tasks/all` | Get all tasks across all sessions |
| `POST` | `/api/tasks/:session_id/:task_id/note` | Add a note to a task |
| `DELETE` | `/api/tasks/:session_id/:task_id` | Delete a task |
| `GET` | `/api/events` | Server-Sent Events (SSE) stream |
| `GET` | `/api/ws` | WebSocket connection |
| `GET` | `/api/theme` | Get current theme |
| `POST` | `/api/theme` | Set theme |
| `GET` | `/api/features/*` | Feature management (if DB enabled) |

### Response Formats

**Sessions List:**
```json
{
  "sessions": [
    {
      "sessionId": "abc-123",
      "customTitle": "OAuth Implementation",
      "slug": "oauth-impl",
      "projectPath": "/Users/me/project",
      "taskCount": 5
    }
  ]
}
```

**Tasks:**
```json
{
  "tasks": [
    {
      "id": "1",
      "subject": "Implement JWT auth",
      "description": "Add token generation and validation",
      "status": "in_progress",
      "owner": "claude",
      "blocks": ["2"],
      "blockedBy": []
    }
  ]
}
```

**SSE Events:**
```json
{"type": "connected"}
{"type": "update", "event": "add", "sessionId": "abc-123", "file": "task-1.json"}
{"type": "metadata-update"}
```

## Performance Features

### Metadata Caching

Session metadata is expensive to compute (requires reading many JSONL files). The server caches it:

```rust
// Cache is automatically managed
// Refreshed every 10 seconds when accessed
// Double-checked locking prevents duplicate refreshes

let metadata = get_metadata(&state).await;
// Fast: uses cache if less than 10s old
// Automatic: refreshes if stale
```

### File Watching

Uses the `notify` crate with debouncing to avoid duplicate events:

```rust
// Watches recursively:
// - ~/.claude/tasks/**/*.json
// - ~/.claude/projects/**/*.jsonl

// Broadcasts to all SSE and WebSocket clients instantly
// Zero polling - instant updates
```

### Async Everything

- All I/O is async (tokio runtime)
- File reading is non-blocking
- Database queries are non-blocking
- Broadcast channel is lock-free

### Response Times

On a modern laptop:
- Static file serving: ~0.6ms
- Session list (with cache): ~1.2ms
- Task queries: ~0.6ms
- SSE connection setup: ~1ms
- WebSocket upgrade: ~1ms

## Configuration

### Environment Variables

```bash
# Server port (default: 3456)
PORT=8080

# Tasks directory (default: ~/.claude/tasks)
TASKS_DIR=/custom/path/tasks

# Projects directory (default: ~/.claude/projects)
PROJECTS_DIR=/custom/path/projects

# Public files directory (default: ./public)
PUBLIC_DIR=/path/to/frontend

# Open browser on startup (default: false)
OPEN_BROWSER=true
```

### Programmatic Configuration

```rust
use flow_core::AgentConfig;
use std::path::PathBuf;

let config = AgentConfig {
    port: 3000,
    tasks_dir: Some(PathBuf::from("/tmp/tasks")),
    projects_dir: Some(PathBuf::from("/tmp/projects")),
    public_dir: Some(PathBuf::from("./dist")),
    open_browser: false,
    db_path: Some(PathBuf::from("features.db")), // Optional
    ..Default::default()
};
```

## File Watcher Details

### Tasks Directory Watching

```
~/.claude/tasks/
  └── session-abc-123/
      ├── task-1.json    ← Watched
      ├── task-2.json    ← Watched
      └── task-3.json    ← Watched
```

Events generated:
- File created → `{"type": "update", "event": "add", "sessionId": "...", "file": "task-1.json"}`
- File modified → `{"type": "update", "event": "change", "sessionId": "...", "file": "task-1.json"}`

### Projects Directory Watching

```
~/.claude/projects/
  └── my-project/
      ├── session-abc.jsonl           ← Watched
      └── sessions-index.json         ← Watched (future)
```

Events generated:
- JSONL modified → `{"type": "metadata-update"}`

### Handling Missing Directories

If `~/.claude/tasks` doesn't exist yet, the watcher watches the parent (`~/.claude`) with non-recursive mode. When `tasks/` is created, it starts watching recursively.

## Testing

```bash
# Run all tests
cargo test -p flow-server

# Run with output
cargo test -p flow-server -- --nocapture

# Test specific module
cargo test -p flow-server routes::

# Integration test: start a test server
cargo run -p flow-server -- --port 9999
```

### Manual Testing

```bash
# Start server
cargo run -p flow-server

# In another terminal, test SSE
curl -N http://localhost:3456/api/events

# Test API
curl http://localhost:3456/api/sessions | jq

# Create a task file and watch SSE update
echo '{"id":"1","subject":"Test"}' > ~/.claude/tasks/test/1.json
```

## WebSocket vs SSE

### When to use SSE (Server-Sent Events)

- ✅ **Unidirectional**: Server to client only
- ✅ **Automatic reconnection**: Browser handles it
- ✅ **Text-based**: Simple JSON messages
- ✅ **Firewall-friendly**: Uses HTTP
- ✅ **Use case**: Task updates, file changes

### When to use WebSocket

- ✅ **Bidirectional**: Client can send messages
- ✅ **Binary support**: Can send binary data
- ✅ **Lower latency**: No HTTP overhead
- ✅ **Full-duplex**: Simultaneous send/receive
- ✅ **Use case**: Agent status updates, chat, real-time collaboration

## Error Handling

```rust
use flow_server::AppError;
use axum::{http::StatusCode, Json};

// Custom error type with automatic conversion to HTTP responses
pub enum AppError {
    NotFound(String),
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    DatabaseError(String),
}

// Automatically converts to appropriate HTTP status codes
// NotFound → 404
// IoError → 500
// JsonError → 400
// DatabaseError → 500
```

## Related Crates

- **[flow-core](../flow-core/README.md)**: Core types and configuration
- **[flow-db](../flow-db/README.md)**: Database backend (optional)
- **[flow-resolver](../flow-resolver/README.md)**: Dependency sorting for API responses
- **[flow-tui](../flow-tui/README.md)**: Terminal UI alternative interface

[Back to main README](../../README.md)
