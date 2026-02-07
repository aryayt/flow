use flow_core::Feature;
use std::collections::{HashMap, HashSet, VecDeque};

/// Compute scheduling scores for features.
/// Score = `unblock_count` * 1000.0 + `depth_score` * 100.0 + `priority_factor` * 10.0
///
/// - `unblock_count`: BFS forward counting how many features this transitively unblocks
/// - `depth_score`: position in reverse topological order, normalized 0.0-1.0
/// - `priority_factor`: feature.priority normalized 0.0-1.0
pub fn compute_scores(features: &[Feature]) -> HashMap<i64, f64> {
    if features.is_empty() {
        return HashMap::new();
    }

    // Build reverse adjacency list: dep_id -> list of features that depend on it (dependents)
    let mut dependents: HashMap<i64, Vec<i64>> = HashMap::new();
    let feature_ids: HashSet<i64> = features.iter().map(|f| f.id).collect();

    for feature in features {
        for &dep_id in &feature.dependencies {
            if feature_ids.contains(&dep_id) {
                dependents.entry(dep_id).or_default().push(feature.id);
            }
        }
    }

    // Compute unblock counts via BFS
    let mut unblock_counts: HashMap<i64, f64> = HashMap::new();
    for feature in features {
        let count = count_transitive_unblocks(feature.id, &dependents, &feature_ids);
        #[allow(clippy::cast_precision_loss)]
        let count_f64 = count as f64;
        unblock_counts.insert(feature.id, count_f64);
    }

    // Compute topological order for depth scores
    let depth_scores = compute_depth_scores(features);

    // Normalize priorities
    let priorities: Vec<i32> = features.iter().map(|f| f.priority).collect();
    let min_priority = *priorities.iter().min().unwrap_or(&0);
    let max_priority = *priorities.iter().max().unwrap_or(&0);
    let priority_range = f64::from(max_priority - min_priority);

    let mut scores = HashMap::new();
    for feature in features {
        let unblock = *unblock_counts.get(&feature.id).unwrap_or(&0.0);
        let depth = *depth_scores.get(&feature.id).unwrap_or(&0.0);
        let priority_factor = if priority_range > 0.0 {
            // Lower priority value = more important = higher factor
            1.0 - f64::from(feature.priority - min_priority) / priority_range
        } else {
            0.0
        };

        let score = unblock * 1000.0 + depth * 100.0 + priority_factor * 10.0;
        scores.insert(feature.id, score);
    }

    scores
}

/// Count how many features this feature transitively unblocks using BFS.
fn count_transitive_unblocks(
    feature_id: i64,
    dependents: &HashMap<i64, Vec<i64>>,
    all_ids: &HashSet<i64>,
) -> usize {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(feature_id);
    visited.insert(feature_id);

    while let Some(node_id) = queue.pop_front() {
        if let Some(deps) = dependents.get(&node_id) {
            for &dep_id in deps {
                if all_ids.contains(&dep_id) && !visited.contains(&dep_id) {
                    visited.insert(dep_id);
                    queue.push_back(dep_id);
                }
            }
        }
    }

    // Don't count the feature itself
    visited.len() - 1
}

/// Compute depth scores based on topological order.
/// Features earlier in topo order (no deps) get score 1.0, later ones get lower scores.
fn compute_depth_scores(features: &[Feature]) -> HashMap<i64, f64> {
    // Use our own topological sort
    let Ok(sorted) = crate::topo::topological_sort(features) else {
        // If there's a cycle, just return zeros
        return features.iter().map(|f| (f.id, 0.0)).collect();
    };

    let mut scores = HashMap::new();
    let len = sorted.len();

    if len == 0 {
        return scores;
    }

    for (idx, feature) in sorted.iter().enumerate() {
        // First in topo order gets 1.0, last gets 0.0
        let score = if len == 1 {
            1.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let idx_f64 = idx as f64;
            #[allow(clippy::cast_precision_loss)]
            let len_minus_1_f64 = (len - 1) as f64;
            1.0 - (idx_f64 / len_minus_1_f64)
        };
        scores.insert(feature.id, score);
    }

    scores
}

/// Check if all dependencies of a feature are satisfied (passes == true).
pub fn are_dependencies_satisfied(feature: &Feature, all_features: &[Feature]) -> bool {
    if feature.dependencies.is_empty() {
        return true;
    }

    let feature_map: HashMap<i64, &Feature> = all_features.iter().map(|f| (f.id, f)).collect();

    feature
        .dependencies
        .iter()
        .all(|dep_id| feature_map.get(dep_id).is_some_and(|dep| dep.passes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_feature(id: i64, priority: i32, dependencies: Vec<i64>, passes: bool) -> Feature {
        Feature {
            id,
            priority,
            category: String::new(),
            name: format!("Feature {id}"),
            description: String::new(),
            steps: Vec::new(),
            passes,
            in_progress: false,
            dependencies,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_single_feature_score() {
        let features = vec![make_feature(1, 100, vec![], false)];
        let scores = compute_scores(&features);
        assert_eq!(scores.len(), 1);
        // Single feature: unblock=0, depth=1.0, priority_factor=0 (only one priority)
        // Score = 0*1000 + 1.0*100 + 0*10 = 100.0
        assert!((scores[&1] - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_linear_chain_unblock_counts() {
        // A(1) -> B(2) -> C(3)
        // A unblocks B and C (count=2)
        // B unblocks C (count=1)
        // C unblocks none (count=0)
        let features = vec![
            make_feature(1, 0, vec![], false),
            make_feature(2, 0, vec![1], false),
            make_feature(3, 0, vec![2], false),
        ];
        let scores = compute_scores(&features);

        // A should have highest unblock count (2)
        // B should have medium unblock count (1)
        // C should have lowest unblock count (0)
        let unblock_a = scores[&1] / 1000.0;
        let unblock_b = scores[&2] / 1000.0;
        let unblock_c = scores[&3] / 1000.0;

        assert!(unblock_a > unblock_b);
        assert!(unblock_b > unblock_c);
        assert!((unblock_a.floor() - 2.0).abs() < f64::EPSILON);
        assert!((unblock_b.floor() - 1.0).abs() < f64::EPSILON);
        assert!((unblock_c.floor() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dependencies_satisfied_all_passing() {
        let features = vec![
            make_feature(1, 0, vec![], true),
            make_feature(2, 0, vec![], true),
            make_feature(3, 0, vec![1, 2], false),
        ];
        let feature_3 = &features[2];
        assert!(are_dependencies_satisfied(feature_3, &features));
    }

    #[test]
    fn test_dependencies_satisfied_some_failing() {
        let features = vec![
            make_feature(1, 0, vec![], true),
            make_feature(2, 0, vec![], false), // Not passing
            make_feature(3, 0, vec![1, 2], false),
        ];
        let feature_3 = &features[2];
        assert!(!are_dependencies_satisfied(feature_3, &features));
    }

    #[test]
    fn test_dependencies_satisfied_missing_dep() {
        let features = vec![
            make_feature(1, 0, vec![], true),
            make_feature(3, 0, vec![1, 99], false), // 99 doesn't exist
        ];
        let feature_3 = &features[1];
        assert!(!are_dependencies_satisfied(feature_3, &features));
    }

    #[test]
    fn test_dependencies_satisfied_no_deps() {
        let feature = make_feature(1, 0, vec![], false);
        assert!(are_dependencies_satisfied(&feature, &[]));
    }

    #[test]
    fn test_depth_score_ordering() {
        // A -> B -> C
        // A should get depth=1.0 (first in topo order)
        // C should get depth=0.0 (last in topo order)
        let features = vec![
            make_feature(1, 0, vec![], false),
            make_feature(2, 0, vec![1], false),
            make_feature(3, 0, vec![2], false),
        ];
        let scores = compute_depth_scores(&features);
        assert!((scores[&1] - 1.0).abs() < f64::EPSILON); // First
        assert!((scores[&3] - 0.0).abs() < f64::EPSILON); // Last
        assert!(scores[&2] > 0.0 && scores[&2] < 1.0); // Middle
    }

    #[test]
    fn test_priority_normalization() {
        // Two features, different priorities, no deps
        // Lower priority value = more important = higher score
        let features = vec![
            make_feature(1, 10, vec![], false),
            make_feature(2, 100, vec![], false),
        ];
        let scores = compute_scores(&features);

        // Feature 1 (priority 10) should score higher than Feature 2 (priority 100)
        // because lower priority value = more important
        assert!(scores[&1] > scores[&2]);

        // The difference should be at least the priority component (10.0)
        let diff = scores[&1] - scores[&2];
        assert!(
            diff >= 10.0,
            "Difference was {diff}, expected at least 10.0"
        );
    }

    #[test]
    fn test_diamond_unblock_counts() {
        // A -> B, A -> C, B -> D, C -> D
        // A unblocks B, C, D (count=3)
        // B unblocks D (count=1)
        // C unblocks D (count=1)
        // D unblocks none (count=0)
        let features = vec![
            make_feature(1, 0, vec![], false),
            make_feature(2, 0, vec![1], false),
            make_feature(3, 0, vec![1], false),
            make_feature(4, 0, vec![2, 3], false),
        ];
        let scores = compute_scores(&features);

        let unblock_a = (scores[&1] / 1000.0).floor();
        let unblock_b = (scores[&2] / 1000.0).floor();
        let unblock_c = (scores[&3] / 1000.0).floor();
        let unblock_d = (scores[&4] / 1000.0).floor();

        assert!((unblock_a - 3.0).abs() < f64::EPSILON);
        assert!((unblock_b - 1.0).abs() < f64::EPSILON);
        assert!((unblock_c - 1.0).abs() < f64::EPSILON);
        assert!((unblock_d - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_empty_features() {
        let scores = compute_scores(&[]);
        assert_eq!(scores.len(), 0);
    }

    #[test]
    fn test_are_dependencies_satisfied_empty_deps() {
        let feature = make_feature(1, 0, vec![], false);
        let features = vec![feature.clone()];
        assert!(are_dependencies_satisfied(&feature, &features));
    }
}
