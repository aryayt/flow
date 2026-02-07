use flow_core::{Feature, FlowError};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A wrapper for heap ordering that prioritizes by (highest priority first, lowest id for ties).
#[derive(Debug, Clone, Eq, PartialEq)]
struct HeapNode {
    priority: i32,
    id: i64,
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower priority value = more important (matches DB ORDER BY priority ASC)
        // BinaryHeap is max-heap, so reverse the priority comparison
        match other.priority.cmp(&self.priority) {
            Ordering::Equal => other.id.cmp(&self.id), // Lower ID wins ties
            ord => ord,
        }
    }
}

impl PartialOrd for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Sort features in dependency-safe execution order using Kahn's algorithm.
/// Returns error with cycle node IDs if cycles are detected.
///
/// Features are ordered such that:
/// 1. Dependencies always come before dependents
/// 2. Among features at the same "depth", higher priority features come first
/// 3. Ties are broken by lowest ID
///
/// # Panics
/// Panics if the internal in-degree map is corrupted (should never happen with valid input).
pub fn topological_sort(features: &[Feature]) -> Result<Vec<Feature>, FlowError> {
    if features.is_empty() {
        return Ok(Vec::new());
    }

    // Build a map from id -> Feature for quick lookup
    let feature_map: HashMap<i64, &Feature> = features.iter().map(|f| (f.id, f)).collect();
    let feature_ids: HashSet<i64> = feature_map.keys().copied().collect();

    // Build adjacency list: dep_id -> list of features that depend on it (dependents)
    let mut dependents: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut in_degree: HashMap<i64, usize> = HashMap::new();

    // Initialize in-degree for all features
    for &id in &feature_ids {
        in_degree.insert(id, 0);
    }

    // Build the graph
    for feature in features {
        for &dep_id in &feature.dependencies {
            // Only include dependencies that exist in our feature set
            if feature_ids.contains(&dep_id) {
                dependents.entry(dep_id).or_default().push(feature.id);
                *in_degree
                    .get_mut(&feature.id)
                    .expect("in_degree initialized for all feature IDs") += 1;
            }
        }
    }

    // Initialize heap with all zero in-degree features
    let mut heap = BinaryHeap::new();
    for (&id, &degree) in &in_degree {
        if degree == 0 {
            let feature = feature_map[&id];
            heap.push(HeapNode {
                priority: feature.priority,
                id,
            });
        }
    }

    let mut result = Vec::new();

    // Process features in dependency order
    while let Some(node) = heap.pop() {
        let feature = feature_map[&node.id];
        result.push(feature.clone());

        // Update dependents
        if let Some(deps) = dependents.get(&node.id) {
            for &dependent_id in deps {
                let degree = in_degree
                    .get_mut(&dependent_id)
                    .expect("dependent_id is a valid feature ID");
                *degree -= 1;
                if *degree == 0 {
                    let dependent = feature_map[&dependent_id];
                    heap.push(HeapNode {
                        priority: dependent.priority,
                        id: dependent_id,
                    });
                }
            }
        }
    }

    // If we didn't process all features, there's a cycle
    if result.len() < features.len() {
        let processed: HashSet<i64> = result.iter().map(|f| f.id).collect();
        let remaining: Vec<String> = feature_ids
            .iter()
            .filter(|id| !processed.contains(id))
            .map(std::string::ToString::to_string)
            .collect();
        return Err(FlowError::CycleDetected(format!(
            "Cycle detected involving features: [{}]",
            remaining.join(", ")
        )));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_feature(id: i64, priority: i32, dependencies: Vec<i64>) -> Feature {
        Feature {
            id,
            priority,
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
    fn test_empty_input() {
        let result = topological_sort(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_single_feature_no_deps() {
        let features = vec![make_feature(1, 0, vec![])];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, 1);
    }

    #[test]
    fn test_linear_chain() {
        // A(1) -> B(2) -> C(3)
        let features = vec![
            make_feature(3, 0, vec![2]),
            make_feature(2, 0, vec![1]),
            make_feature(1, 0, vec![]),
        ];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 2);
        assert_eq!(result[2].id, 3);
    }

    #[test]
    fn test_diamond_dependency() {
        // A(1) -> B(2), A(1) -> C(3), B(2) -> D(4), C(3) -> D(4)
        let features = vec![
            make_feature(1, 0, vec![]),
            make_feature(2, 0, vec![1]),
            make_feature(3, 0, vec![1]),
            make_feature(4, 0, vec![2, 3]),
        ];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].id, 1); // A first
        assert_eq!(result[3].id, 4); // D last
                                     // B and C can be in either order, but both come after A and before D
        let b_pos = result.iter().position(|f| f.id == 2).unwrap();
        let c_pos = result.iter().position(|f| f.id == 3).unwrap();
        assert!(b_pos > 0 && b_pos < 3);
        assert!(c_pos > 0 && c_pos < 3);
    }

    #[test]
    fn test_cycle_detection() {
        // A(1) -> B(2) -> A(1)
        let features = vec![make_feature(1, 0, vec![2]), make_feature(2, 0, vec![1])];
        let result = topological_sort(&features);
        assert!(result.is_err());
        if let Err(FlowError::CycleDetected(msg)) = result {
            assert!(msg.contains('1'));
            assert!(msg.contains('2'));
        } else {
            panic!("Expected CycleDetected error");
        }
    }

    #[test]
    fn test_priority_ordering() {
        // Two features with no deps, different priorities
        // Lower priority value = more important (like queue position: 1 = first)
        let features = vec![make_feature(1, 10, vec![]), make_feature(2, 100, vec![])];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1); // Lower priority value (10) = more important
        assert_eq!(result[1].id, 2);
    }

    #[test]
    fn test_priority_with_same_depth() {
        // A(1) -> B(2), A(1) -> C(3)
        // B has priority 50, C has priority 100
        // B should come before C since lower priority value = more important
        let features = vec![
            make_feature(1, 0, vec![]),
            make_feature(2, 50, vec![1]),
            make_feature(3, 100, vec![1]),
        ];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 2); // Lower priority value = more important
        assert_eq!(result[2].id, 3);
    }

    #[test]
    fn test_deps_pointing_to_non_existent_features() {
        // Feature 2 depends on feature 99 which doesn't exist
        // Should ignore the non-existent dependency and succeed
        let features = vec![make_feature(1, 0, vec![]), make_feature(2, 0, vec![99, 1])];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[1].id, 2);
    }

    #[test]
    fn test_id_tiebreaker() {
        // Two features, same priority, no deps
        // Lower ID should come first
        let features = vec![make_feature(5, 100, vec![]), make_feature(3, 100, vec![])];
        let result = topological_sort(&features).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 3); // Lower ID
        assert_eq!(result[1].id, 5);
    }
}
