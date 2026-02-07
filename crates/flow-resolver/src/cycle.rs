use flow_core::Feature;
use std::collections::{HashMap, HashSet};

const MAX_DEPTH: usize = 50;

/// Check if adding a dependency from `feature_id` -> `dep_id` would create a cycle.
/// Does DFS from `dep_id` to see if `feature_id` is reachable. Depth limit: 50.
pub fn would_create_cycle(feature_id: i64, new_dep_id: i64, features: &[Feature]) -> bool {
    // Trivial self-loop check
    if feature_id == new_dep_id {
        return true;
    }

    // Build adjacency list: id -> list of dependencies
    let mut deps_map: HashMap<i64, Vec<i64>> = HashMap::new();
    for feature in features {
        deps_map.insert(feature.id, feature.dependencies.clone());
    }

    // Add the hypothetical new dependency
    deps_map
        .entry(feature_id)
        .or_default()
        .push(new_dep_id);

    // DFS from new_dep_id following dependency edges
    // If we reach feature_id, adding this dep would create a cycle
    let mut visited = HashSet::new();
    let mut stack = vec![(new_dep_id, 0)]; // (node_id, depth)

    while let Some((node_id, depth)) = stack.pop() {
        if node_id == feature_id {
            return true; // Cycle detected
        }

        if depth >= MAX_DEPTH {
            continue; // Don't recurse deeper
        }

        if visited.contains(&node_id) {
            continue; // Already processed
        }

        visited.insert(node_id);

        // Push all dependencies of this node
        if let Some(deps) = deps_map.get(&node_id) {
            for &dep_id in deps {
                if !visited.contains(&dep_id) {
                    stack.push((dep_id, depth + 1));
                }
            }
        }
    }

    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    White, // Unvisited
    Gray,  // In progress
    Black, // Done
}

/// Find all cycles in the dependency graph.
/// Returns a Vec of cycles, where each cycle is a Vec of feature IDs.
pub fn find_cycles(features: &[Feature]) -> Vec<Vec<i64>> {
    // Build adjacency list: id -> list of dependencies
    let mut deps_map: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut all_ids = HashSet::new();

    for feature in features {
        all_ids.insert(feature.id);
        deps_map.insert(feature.id, feature.dependencies.clone());
    }

    let mut colors: HashMap<i64, Color> = all_ids.iter().map(|&id| (id, Color::White)).collect();
    let mut cycles = Vec::new();
    let mut path = Vec::new();

    for &start_id in &all_ids {
        if colors[&start_id] == Color::White {
            dfs_find_cycles(
                start_id,
                &deps_map,
                &mut colors,
                &mut path,
                &mut cycles,
                &all_ids,
            );
        }
    }

    cycles
}

fn dfs_find_cycles(
    node_id: i64,
    deps_map: &HashMap<i64, Vec<i64>>,
    colors: &mut HashMap<i64, Color>,
    path: &mut Vec<i64>,
    cycles: &mut Vec<Vec<i64>>,
    all_ids: &HashSet<i64>,
) {
    colors.insert(node_id, Color::Gray);
    path.push(node_id);

    if let Some(deps) = deps_map.get(&node_id) {
        for &dep_id in deps {
            // Only follow edges to nodes that exist in our feature set
            if !all_ids.contains(&dep_id) {
                continue;
            }

            match colors.get(&dep_id) {
                Some(Color::Gray) => {
                    // Found a cycle! Extract it from path
                    if let Some(cycle_start) = path.iter().position(|&id| id == dep_id) {
                        let cycle = path[cycle_start..].to_vec();
                        // Only add if we haven't seen this cycle before (different rotation)
                        if !is_duplicate_cycle(&cycle, cycles) {
                            cycles.push(cycle);
                        }
                    }
                }
                Some(Color::White) => {
                    dfs_find_cycles(dep_id, deps_map, colors, path, cycles, all_ids);
                }
                _ => {} // Black - already processed
            }
        }
    }

    path.pop();
    colors.insert(node_id, Color::Black);
}

fn is_duplicate_cycle(cycle: &[i64], existing_cycles: &[Vec<i64>]) -> bool {
    for existing in existing_cycles {
        if is_same_cycle(cycle, existing) {
            return true;
        }
    }
    false
}

fn is_same_cycle(a: &[i64], b: &[i64]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    // Check if b is a rotation of a
    for i in 0..b.len() {
        let mut matches = true;
        for j in 0..a.len() {
            if a[j] != b[(i + j) % b.len()] {
                matches = false;
                break;
            }
        }
        if matches {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_feature(id: i64, dependencies: Vec<i64>) -> Feature {
        Feature {
            id,
            priority: 0,
            category: String::new(),
            name: format!("Feature {id}"),
            description: String::new(),
            steps: Vec::new(),
            passes: false,
            in_progress: false,
            dependencies,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_self_loop_detection() {
        let features = vec![make_feature(1, vec![])];
        assert!(would_create_cycle(1, 1, &features));
    }

    #[test]
    fn test_direct_cycle() {
        // A -> B, want to add B -> A
        let features = vec![make_feature(1, vec![2]), make_feature(2, vec![])];
        assert!(would_create_cycle(2, 1, &features));
    }

    #[test]
    fn test_transitive_cycle() {
        // A -> B -> C, want to add C -> A
        let features = vec![
            make_feature(1, vec![2]),
            make_feature(2, vec![3]),
            make_feature(3, vec![]),
        ];
        assert!(would_create_cycle(3, 1, &features));
    }

    #[test]
    fn test_no_cycle_linear_chain() {
        // A -> B -> C, want to add D -> A (no cycle)
        let features = vec![
            make_feature(1, vec![2]),
            make_feature(2, vec![3]),
            make_feature(3, vec![]),
            make_feature(4, vec![]),
        ];
        assert!(!would_create_cycle(4, 1, &features));
    }

    #[test]
    fn test_depth_limit() {
        // Create a long chain and try to detect cycle at the end
        let mut features = Vec::new();
        for i in 1..=60 {
            if i < 60 {
                features.push(make_feature(i, vec![i + 1]));
            } else {
                features.push(make_feature(i, vec![]));
            }
        }
        // Would create cycle 60 -> 1, but depth limit should prevent detection
        // Actually, depth limit is 50, so we can't traverse the full chain
        // This means we WON'T detect the cycle if the chain is too long
        assert!(!would_create_cycle(60, 1, &features));
    }

    #[test]
    fn test_find_cycles_simple() {
        // A -> B -> A
        let features = vec![make_feature(1, vec![2]), make_feature(2, vec![1])];
        let cycles = find_cycles(&features);
        assert_eq!(cycles.len(), 1);
        assert!(cycles[0].contains(&1));
        assert!(cycles[0].contains(&2));
    }

    #[test]
    fn test_find_cycles_three_node() {
        // A -> B -> C -> A
        let features = vec![
            make_feature(1, vec![2]),
            make_feature(2, vec![3]),
            make_feature(3, vec![1]),
        ];
        let cycles = find_cycles(&features);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].len(), 3);
    }

    #[test]
    fn test_find_cycles_no_cycles() {
        // Linear chain: A -> B -> C
        let features = vec![
            make_feature(1, vec![2]),
            make_feature(2, vec![3]),
            make_feature(3, vec![]),
        ];
        let cycles = find_cycles(&features);
        assert_eq!(cycles.len(), 0);
    }

    #[test]
    fn test_find_cycles_multiple_independent() {
        // Two independent cycles: (A -> B -> A) and (C -> D -> C)
        let features = vec![
            make_feature(1, vec![2]),
            make_feature(2, vec![1]),
            make_feature(3, vec![4]),
            make_feature(4, vec![3]),
        ];
        let cycles = find_cycles(&features);
        assert_eq!(cycles.len(), 2);
    }

    #[test]
    fn test_find_cycles_with_non_existent_deps() {
        // A -> B -> 99 (99 doesn't exist, so no cycle)
        let features = vec![make_feature(1, vec![2]), make_feature(2, vec![99])];
        let cycles = find_cycles(&features);
        assert_eq!(cycles.len(), 0);
    }

    #[test]
    fn test_would_create_cycle_complex() {
        // Diamond: A -> B, A -> C, B -> D, C -> D
        // Adding D -> A would create a cycle
        let features = vec![
            make_feature(1, vec![2, 3]),
            make_feature(2, vec![4]),
            make_feature(3, vec![4]),
            make_feature(4, vec![]),
        ];
        assert!(would_create_cycle(4, 1, &features));
    }

    #[test]
    fn test_would_not_create_cycle_safe_addition() {
        // A -> B, C standalone
        // Adding C -> B is safe
        let features = vec![
            make_feature(1, vec![2]),
            make_feature(2, vec![]),
            make_feature(3, vec![]),
        ];
        assert!(!would_create_cycle(3, 2, &features));
    }
}
