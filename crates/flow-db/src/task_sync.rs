use flow_core::{Feature, Task};

/// Convert a Feature to a Claude Code Task format.
pub fn feature_to_task(feature: &Feature, _session_id: &str) -> Task {
    let status = if feature.passes {
        "completed"
    } else if feature.in_progress {
        "in_progress"
    } else {
        "pending"
    };

    let active_form = if feature.in_progress {
        format!("Working on: {}", feature.name)
    } else {
        String::new()
    };

    // Convert dependency IDs to strings
    let blocked_by: Vec<String> = feature
        .dependencies
        .iter()
        .map(std::string::ToString::to_string)
        .collect();

    Task {
        id: feature.id.to_string(),
        subject: feature.name.clone(),
        description: feature.description.clone(),
        active_form,
        status: status.to_string(),
        owner: "flow".to_string(),
        blocks: vec![],
        blocked_by,
    }
}

/// Convert a Claude Code Task to a Feature (best effort).
/// Note: This is lossy since Tasks don't have all Feature fields.
pub fn task_to_feature(task: &Task) -> Feature {
    let id = task.id.parse::<i64>().unwrap_or(0);

    let passes = task.status == "completed";
    let in_progress = task.status == "in_progress";

    // Parse blockedBy as dependency IDs
    let dependencies: Vec<i64> = task
        .blocked_by
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();

    Feature {
        id,
        priority: 0, // Unknown from task
        category: String::new(), // Unknown from task
        name: task.subject.clone(),
        description: task.description.clone(),
        steps: vec![],
        passes,
        in_progress,
        dependencies,
        created_at: None,
        updated_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_to_task() {
        let feature = Feature {
            id: 42,
            priority: 1,
            category: "Backend".to_string(),
            name: "Implement Auth".to_string(),
            description: "Add OAuth support".to_string(),
            steps: vec!["Step 1".to_string()],
            passes: false,
            in_progress: true,
            dependencies: vec![10, 20],
            created_at: Some("2025-01-01".to_string()),
            updated_at: Some("2025-01-02".to_string()),
        };

        let task = feature_to_task(&feature, "session-123");

        assert_eq!(task.id, "42");
        assert_eq!(task.subject, "Implement Auth");
        assert_eq!(task.description, "Add OAuth support");
        assert_eq!(task.status, "in_progress");
        assert!(task.active_form.contains("Working on"));
        assert_eq!(task.blocked_by, vec!["10", "20"]);
        assert_eq!(task.owner, "flow");
    }

    #[test]
    fn test_feature_to_task_completed() {
        let feature = Feature {
            id: 1,
            priority: 1,
            category: "".to_string(),
            name: "Done".to_string(),
            description: "".to_string(),
            steps: vec![],
            passes: true,
            in_progress: false,
            dependencies: vec![],
            created_at: None,
            updated_at: None,
        };

        let task = feature_to_task(&feature, "session");
        assert_eq!(task.status, "completed");
        assert_eq!(task.active_form, "");
    }

    #[test]
    fn test_feature_to_task_pending() {
        let feature = Feature {
            id: 2,
            priority: 1,
            category: "".to_string(),
            name: "Pending".to_string(),
            description: "".to_string(),
            steps: vec![],
            passes: false,
            in_progress: false,
            dependencies: vec![],
            created_at: None,
            updated_at: None,
        };

        let task = feature_to_task(&feature, "session");
        assert_eq!(task.status, "pending");
    }

    #[test]
    fn test_task_to_feature() {
        let task = Task {
            id: "123".to_string(),
            subject: "Build API".to_string(),
            description: "REST API".to_string(),
            active_form: "Working...".to_string(),
            status: "in_progress".to_string(),
            owner: "agent".to_string(),
            blocks: vec![],
            blocked_by: vec!["10".to_string(), "20".to_string()],
        };

        let feature = task_to_feature(&task);

        assert_eq!(feature.id, 123);
        assert_eq!(feature.name, "Build API");
        assert_eq!(feature.description, "REST API");
        assert!(feature.in_progress);
        assert!(!feature.passes);
        assert_eq!(feature.dependencies, vec![10, 20]);
    }

    #[test]
    fn test_task_to_feature_completed() {
        let task = Task {
            id: "1".to_string(),
            subject: "Done".to_string(),
            description: "".to_string(),
            active_form: "".to_string(),
            status: "completed".to_string(),
            owner: "".to_string(),
            blocks: vec![],
            blocked_by: vec![],
        };

        let feature = task_to_feature(&task);
        assert!(feature.passes);
        assert!(!feature.in_progress);
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original_feature = Feature {
            id: 5,
            priority: 3,
            category: "UI".to_string(),
            name: "Dashboard".to_string(),
            description: "User dashboard".to_string(),
            steps: vec![],
            passes: false,
            in_progress: true,
            dependencies: vec![1, 2],
            created_at: None,
            updated_at: None,
        };

        let task = feature_to_task(&original_feature, "session");
        let converted_feature = task_to_feature(&task);

        // Core fields should match
        assert_eq!(converted_feature.id, original_feature.id);
        assert_eq!(converted_feature.name, original_feature.name);
        assert_eq!(converted_feature.description, original_feature.description);
        assert_eq!(converted_feature.passes, original_feature.passes);
        assert_eq!(converted_feature.in_progress, original_feature.in_progress);
        assert_eq!(converted_feature.dependencies, original_feature.dependencies);

        // Some fields will be lost (priority, category, steps, timestamps)
        assert_eq!(converted_feature.priority, 0);
        assert_eq!(converted_feature.category, "");
    }
}
