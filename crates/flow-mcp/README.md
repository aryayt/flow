# flow-mcp

Model Context Protocol (MCP) server exposing feature management tools for AI agents.

## What it does

`flow-mcp` is like a toolbox for AI agents. It wraps the Flow feature management system into a set of "tools" that AI assistants (like Claude, GPT-4, or Gemini) can call.

Think of it like this:
- **Without MCP**: AI can only read/write files and run shell commands
- **With MCP**: AI gets structured tools like "create_feature", "get_ready_features", "mark_passing"

It's the bridge that lets AI agents directly interact with your feature database instead of fumbling with JSON files.

## Status

⚠️ **Phase 2 - Planned for Future Implementation**

This crate is currently a placeholder. Full implementation is scheduled for Phase 2 of the Flow project.

## Planned Architecture

```
flow-mcp/
├── lib.rs                - MCP server setup and tool registration
├── tools/
│   ├── mod.rs            - Tool organization
│   ├── features.rs       - Feature CRUD tools
│   ├── dependencies.rs   - Dependency management tools
│   ├── scheduling.rs     - Scheduling and scoring tools
│   └── query.rs          - Query and filtering tools
├── types.rs              - MCP-specific type definitions
├── handlers.rs           - Tool execution handlers
└── server.rs             - JSON-RPC server implementation
```

## Planned Features

### Tool Categories

1. **Feature Management**
   - `create_feature` - Create a new feature
   - `update_feature` - Modify feature properties
   - `delete_feature` - Remove a feature
   - `get_feature` - Retrieve feature details

2. **Dependency Management**
   - `add_dependency` - Add a feature dependency
   - `remove_dependency` - Remove a dependency
   - `check_cycles` - Detect circular dependencies
   - `get_dependency_graph` - Visualize dependencies

3. **Scheduling & Workflow**
   - `get_ready_features` - Find features ready to work on
   - `get_blocked_features` - Find blocked features
   - `compute_priorities` - Calculate scheduling scores
   - `claim_feature` - Atomically claim a feature

4. **Querying**
   - `list_features` - List all features with filters
   - `search_features` - Search by name/description
   - `get_statistics` - Get aggregate stats
   - `get_category_breakdown` - Group by category

## Expected Usage

### Starting the MCP Server

```bash
# Start MCP server on stdio (for Claude Desktop)
flow-mcp --transport stdio

# Start on HTTP (for web clients)
flow-mcp --transport http --port 8080

# With custom database
flow-mcp --db-path /path/to/features.db
```

### Claude Desktop Configuration

```json
{
  "mcpServers": {
    "flow": {
      "command": "flow-mcp",
      "args": ["--transport", "stdio"],
      "env": {
        "DB_PATH": "/Users/me/.flow/features.db"
      }
    }
  }
}
```

### Agent Using MCP Tools

```python
# Hypothetical usage in an agent
from mcp import Client

client = Client()

# List features ready to work on
ready = client.call_tool("get_ready_features", {})
print(f"Found {len(ready)} features ready to work on")

# Claim a feature
feature = client.call_tool("claim_feature", {
    "feature_id": ready[0]["id"]
})
print(f"Claimed: {feature['name']}")

# Mark as passing when done
client.call_tool("mark_passing", {
    "feature_id": feature["id"]
})
```

### TypeScript/JavaScript Client

```typescript
import { MCPClient } from '@modelcontextprotocol/sdk';

const client = new MCPClient({
  transport: 'stdio',
  command: 'flow-mcp'
});

// Create a feature with dependencies
const feature = await client.callTool('create_feature', {
  name: 'User Authentication',
  description: 'Implement JWT-based auth',
  priority: 100,
  category: 'Backend',
  steps: ['Create user model', 'Add login endpoint'],
  dependencies: [1, 2] // Depends on features 1 and 2
});

console.log(`Created feature #${feature.id}`);
```

## Planned Tool Schemas

### create_feature

```json
{
  "name": "create_feature",
  "description": "Create a new feature in the database",
  "inputSchema": {
    "type": "object",
    "properties": {
      "name": {
        "type": "string",
        "description": "Feature name"
      },
      "description": {
        "type": "string",
        "description": "Detailed description"
      },
      "priority": {
        "type": "integer",
        "description": "Priority (lower = more important)"
      },
      "category": {
        "type": "string",
        "description": "Category (e.g., 'Backend', 'Frontend')"
      },
      "steps": {
        "type": "array",
        "items": { "type": "string" },
        "description": "Implementation steps"
      },
      "dependencies": {
        "type": "array",
        "items": { "type": "integer" },
        "description": "Feature IDs this depends on"
      }
    },
    "required": ["name", "description", "category"]
  }
}
```

### get_ready_features

```json
{
  "name": "get_ready_features",
  "description": "Get features ready to work on (dependencies satisfied, not in progress)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "limit": {
        "type": "integer",
        "description": "Maximum number of features to return",
        "default": 10
      },
      "category": {
        "type": "string",
        "description": "Filter by category (optional)"
      }
    }
  }
}
```

### claim_feature

```json
{
  "name": "claim_feature",
  "description": "Atomically claim a feature for work (prevents race conditions)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "feature_id": {
        "type": "integer",
        "description": "Feature ID to claim"
      }
    },
    "required": ["feature_id"]
  }
}
```

### compute_priorities

```json
{
  "name": "compute_priorities",
  "description": "Calculate scheduling scores for all features",
  "inputSchema": {
    "type": "object",
    "properties": {
      "sort_by": {
        "type": "string",
        "enum": ["score", "priority", "unblock_count"],
        "default": "score",
        "description": "How to sort results"
      }
    }
  }
}
```

## Integration Examples

### Multi-Agent Workflow

```python
# Agent 1: Backend specialist
backend_features = mcp.call_tool("list_features", {
    "category": "Backend",
    "status": "ready"
})

for feature in backend_features[:3]:  # Work on top 3
    claimed = mcp.call_tool("claim_feature", {"feature_id": feature["id"]})
    # Do work...
    mcp.call_tool("mark_passing", {"feature_id": claimed["id"]})

# Agent 2: Frontend specialist
frontend_features = mcp.call_tool("get_ready_features", {})
frontend_features = [f for f in frontend_features if f["category"] == "Frontend"]
# ... work on frontend features
```

### Dependency Validation

```python
# Before adding a dependency, check for cycles
will_create_cycle = mcp.call_tool("check_cycles", {
    "feature_id": 42,
    "new_dependency_id": 10
})

if will_create_cycle:
    print("ERROR: Would create circular dependency!")
else:
    mcp.call_tool("add_dependency", {
        "feature_id": 42,
        "dependency_id": 10
    })
```

### Smart Scheduling

```python
# Get prioritized work queue
priorities = mcp.call_tool("compute_priorities", {
    "sort_by": "score"
})

print("Work on features in this order:")
for i, feature in enumerate(priorities[:10], 1):
    print(f"{i}. {feature['name']} (score: {feature['score']:.1f})")
    print(f"   Unblocks {feature['unblock_count']} others")
```

## Implementation Roadmap

**Phase 2 Tasks:**

1. ✅ MCP protocol implementation (JSON-RPC over stdio/HTTP)
2. ✅ Tool schema definitions
3. ✅ Handler functions connecting to flow-db
4. ✅ Error handling and validation
5. ✅ Integration tests with Claude Code
6. ✅ Documentation and examples

**Estimated Timeline:** 2-3 weeks

## Technical Details

### MCP Protocol

Uses [Model Context Protocol](https://modelcontextprotocol.io/) specification:
- JSON-RPC 2.0 for request/response
- Supports stdio and HTTP transports
- Tool discovery via `tools/list` endpoint
- Structured input/output schemas

### Database Connection

```rust
// Shared database handle across all tool calls
pub struct McpServer {
    db: Arc<flow_db::Database>,
    resolver: Arc<flow_resolver::Resolver>,
}

impl McpServer {
    pub fn new(db_path: &Path) -> Result<Self> {
        Ok(Self {
            db: Arc::new(Database::open(db_path)?),
            resolver: Arc::new(Resolver::new()),
        })
    }

    pub async fn handle_tool_call(&self, name: &str, params: Value) -> Result<Value> {
        match name {
            "create_feature" => self.create_feature(params).await,
            "get_ready_features" => self.get_ready_features(params).await,
            // ... other tools
            _ => Err(McpError::UnknownTool(name.to_string())),
        }
    }
}
```

## Testing

```bash
# Unit tests (when implemented)
cargo test -p flow-mcp

# Integration test with mock AI agent
cargo run -p flow-mcp --example test-agent

# Test with Claude Desktop
# (Add to Claude Desktop config and interact via chat)
```

## Security Considerations

- **Authentication**: MCP servers should validate agent identity
- **Rate limiting**: Prevent abuse by limiting tool calls
- **Input validation**: Strict schema validation on all inputs
- **Audit logging**: Log all tool calls with agent ID and timestamp

## Related Crates

- **[flow-core](../flow-core/README.md)**: Core types used in tool schemas
- **[flow-db](../flow-db/README.md)**: Database backend for tools
- **[flow-resolver](../flow-resolver/README.md)**: Dependency resolution for scheduling tools
- **[flow-orchestrator](../flow-orchestrator/README.md)**: Agent management (uses MCP)

## External Resources

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [MCP SDK (TypeScript)](https://github.com/modelcontextprotocol/sdk)
- [Claude Code MCP Documentation](https://docs.anthropic.com/claude/docs/mcp)

[Back to main README](../../README.md)
