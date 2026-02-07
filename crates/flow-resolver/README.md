# flow-resolver

Dependency resolution, topological sorting, cycle detection, and intelligent scheduling for feature management.

## What it does

Imagine you're building a house. You can't install the roof before building the walls, and you can't build the walls before laying the foundation. `flow-resolver` solves this kind of problem for your features.

It figures out the correct order to work on features based on their dependencies, detects circular dependencies (like "A depends on B, but B depends on A" - which is impossible!), and even calculates priority scores to help you decide what to work on next.

Think of it as a smart project manager that:
- **Sorts tasks** so dependencies always come before the things that need them
- **Catches mistakes** like circular dependencies before they cause problems
- **Recommends priorities** based on which tasks will unblock the most other work

## Architecture

```
flow-resolver/
├── lib.rs      - Public API exports
├── topo.rs     - Topological sorting (Kahn's algorithm)
├── cycle.rs    - Cycle detection (DFS-based)
└── scorer.rs   - Priority scoring and dependency satisfaction
```

### Key Algorithms

1. **Topological Sort** (`topo.rs`): Uses Kahn's algorithm with priority queue
2. **Cycle Detection** (`cycle.rs`): Depth-first search with color marking
3. **Scoring** (`scorer.rs`): Multi-factor scoring (unblock count + depth + priority)

## Usage

### Topological sorting

```rust
use flow_resolver::topological_sort;

// Create some features with dependencies
let features = vec![
    Feature { id: 1, priority: 100, dependencies: vec![], ... },     // Foundation
    Feature { id: 2, priority: 90, dependencies: vec![1], ... },     // Walls
    Feature { id: 3, priority: 80, dependencies: vec![1], ... },     // Plumbing
    Feature { id: 4, priority: 70, dependencies: vec![2, 3], ... },  // Roof
];

// Sort in dependency-safe order
let sorted = topological_sort(&features)?;

// Result: [1, 2, 3, 4] or [1, 3, 2, 4]
// Foundation first, roof last, walls/plumbing in between
for (i, feature) in sorted.iter().enumerate() {
    println!("Step {}: {}", i + 1, feature.name);
}
```

### Cycle detection

```rust
use flow_resolver::{would_create_cycle, find_cycles};

// Check if adding a dependency would create a cycle
let features = vec![
    Feature { id: 1, dependencies: vec![2], ... },
    Feature { id: 2, dependencies: vec![3], ... },
    Feature { id: 3, dependencies: vec![], ... },
];

// Would adding "3 depends on 1" create a cycle?
if would_create_cycle(3, 1, &features) {
    println!("ERROR: Cannot add dependency - would create cycle!");
    // 3 -> 1 -> 2 -> 3 (circular!)
} else {
    println!("Safe to add dependency");
}

// Find all existing cycles in the graph
let cycles = find_cycles(&features);
if !cycles.is_empty() {
    println!("Found {} cycles:", cycles.len());
    for cycle in cycles {
        println!("  Cycle: {:?}", cycle); // e.g., [1, 2, 3]
    }
}
```

### Priority scoring

```rust
use flow_resolver::{compute_scores, are_dependencies_satisfied};

let features = vec![
    Feature { id: 1, priority: 100, dependencies: vec![], passes: true, ... },
    Feature { id: 2, priority: 90, dependencies: vec![1], passes: false, ... },
    Feature { id: 3, priority: 80, dependencies: vec![2], passes: false, ... },
];

// Compute scheduling scores
let scores = compute_scores(&features);

// Sort by score (highest first)
let mut scored_features: Vec<_> = features.iter().collect();
scored_features.sort_by(|a, b| {
    scores[&b.id].partial_cmp(&scores[&a.id]).unwrap()
});

println!("Work on features in this order:");
for feature in scored_features {
    let score = scores[&feature.id];
    println!("  {} (score: {:.1})", feature.name, score);
}

// Check if a feature is ready to work on
let feature_2 = &features[1];
if are_dependencies_satisfied(feature_2, &features) {
    println!("{} is ready to work on!", feature_2.name);
} else {
    println!("{} is blocked by dependencies", feature_2.name);
}
```

### Practical example: Build automation

```rust
use flow_resolver::{topological_sort, compute_scores};

// Features for a build pipeline
let features = vec![
    Feature { id: 1, name: "Install deps", dependencies: vec![], ... },
    Feature { id: 2, name: "Lint code", dependencies: vec![1], ... },
    Feature { id: 3, name: "Run tests", dependencies: vec![1], ... },
    Feature { id: 4, name: "Build app", dependencies: vec![2, 3], ... },
    Feature { id: 5, name: "Deploy", dependencies: vec![4], ... },
];

// Get safe execution order
let build_order = topological_sort(&features)?;

println!("Build pipeline:");
for (step, feature) in build_order.iter().enumerate() {
    println!("  {}. {}", step + 1, feature.name);
}

// Calculate priorities
let scores = compute_scores(&features);
println!("\nMost critical feature: {}",
    build_order.iter()
        .max_by_key(|f| (scores[&f.id] * 100.0) as i64)
        .unwrap()
        .name
);
```

## How Scoring Works

The scoring algorithm uses three factors:

```
Score = (unblock_count × 1000) + (depth_score × 100) + (priority_factor × 10)
```

1. **Unblock Count** (weight: 1000): How many features this transitively unblocks
   - Foundation features that many things depend on get high scores
   - Calculated via BFS through the dependency graph

2. **Depth Score** (weight: 100): Position in topological order
   - Features with no dependencies get score 1.0
   - Features at the end of the chain get score 0.0
   - Normalized between 0.0 and 1.0

3. **Priority Factor** (weight: 10): User-assigned priority
   - Lower priority value = more important (like queue position: 1 is first)
   - Normalized between 0.0 and 1.0

### Example Scores

```
Feature A (no deps, unblocks 3 others, priority=10):
  Score = 3×1000 + 1.0×100 + 1.0×10 = 3110

Feature B (1 dep, unblocks 1 other, priority=50):
  Score = 1×1000 + 0.5×100 + 0.5×10 = 1055

Feature C (2 deps, unblocks 0 others, priority=90):
  Score = 0×1000 + 0.0×100 + 0.2×10 = 2
```

Work on A first, then B, then C.

## API Reference

| Function | Description |
|----------|-------------|
| `topological_sort(features)` | Sort features in dependency-safe order |
| `would_create_cycle(feature_id, new_dep_id, features)` | Check if adding a dependency creates a cycle |
| `find_cycles(features)` | Find all cycles in the dependency graph |
| `compute_scores(features)` | Calculate priority scores for all features |
| `are_dependencies_satisfied(feature, all_features)` | Check if all dependencies are passing |

### Error Handling

```rust
use flow_core::FlowError;

match topological_sort(&features) {
    Ok(sorted) => println!("Safe order: {:?}", sorted),
    Err(FlowError::CycleDetected(msg)) => {
        println!("Cannot sort - cycle detected: {}", msg);
        let cycles = find_cycles(&features);
        println!("Cycles found: {:?}", cycles);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Algorithm Details

### Kahn's Algorithm (Topological Sort)

1. Count in-degree (number of incoming edges) for each node
2. Add all zero-in-degree nodes to a priority queue
3. While queue is not empty:
   - Pop highest priority node
   - Add to result
   - Decrease in-degree of all dependents
   - Add newly-zero-degree dependents to queue
4. If result has fewer nodes than input, a cycle exists

Time complexity: O(V + E) where V = features, E = dependencies

### DFS Cycle Detection

Uses color marking:
- White: Unvisited
- Gray: In progress (on current path)
- Black: Done

If we encounter a gray node, we've found a cycle.

Time complexity: O(V + E)

### Depth Limit

Cycle detection has a depth limit of 50 to prevent infinite loops in pathological cases. For most real-world dependency graphs, this is more than sufficient.

## Testing

```bash
# Run all tests
cargo test -p flow-resolver

# Run with output
cargo test -p flow-resolver -- --nocapture

# Run specific tests
cargo test -p flow-resolver test_cycle_detection
cargo test -p flow-resolver test_priority_ordering
cargo test -p flow-resolver test_linear_chain_unblock_counts
```

## Performance

Benchmarks on typical feature graphs:

- **Topological sort**: ~50μs for 100 features
- **Cycle detection**: ~30μs for 100 features
- **Score computation**: ~100μs for 100 features

All algorithms are O(V + E) which scales linearly with graph size.

## Related Crates

- **[flow-core](../flow-core/README.md)**: Feature type definition
- **[flow-db](../flow-db/README.md)**: Uses resolver for dependency validation
- **[flow-server](../flow-server/README.md)**: Uses resolver for sorting API responses
- **[flow-orchestrator](../flow-orchestrator/README.md)**: Uses scoring for agent scheduling

[Back to main README](../../README.md)
