use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use std::path::Path;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

#[derive(Debug, Clone, Serialize)]
struct SsePayload {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sessionId")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
}

/// Set up file watchers for tasks and projects directories
///
/// # Panics
///
/// May panic if the fallback dummy watcher cannot be created.
#[allow(clippy::cognitive_complexity)]
pub fn setup_file_watcher(
    tasks_dir: &Path,
    projects_dir: &Path,
    tx: broadcast::Sender<String>,
) -> Option<(RecommendedWatcher, RecommendedWatcher)> {
    let tasks_dir_clone = tasks_dir.to_path_buf();

    // Tasks watcher
    let tx_tasks = tx.clone();
    let tasks_watcher_result = notify::recommended_watcher(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                for path in &event.paths {
                    if path.extension().is_some_and(|e| e == "json") {
                        let relative = path
                            .strip_prefix(&tasks_dir_clone)
                            .unwrap_or(path);
                        let session_id = relative
                            .components()
                            .next()
                            .map(|c| c.as_os_str().to_string_lossy().to_string())
                            .unwrap_or_default();
                        let file_name = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();

                        let event_name = if matches!(event.kind, notify::EventKind::Create(_)) {
                            "add"
                        } else {
                            "change"
                        };

                        let payload = SsePayload {
                            event_type: "update".to_string(),
                            event: Some(event_name.to_string()),
                            session_id: Some(session_id),
                            file: Some(file_name),
                        };

                        if let Ok(json) = serde_json::to_string(&payload) {
                            let _ = tx_tasks.send(json);
                        }
                    }
                }
            }
        },
    );

    let mut tasks_watcher = match tasks_watcher_result {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create tasks watcher: {e}");
            return None;
        }
    };

    // Ensure tasks dir exists before watching
    if tasks_dir.exists() {
        if let Err(e) = tasks_watcher.watch(tasks_dir, RecursiveMode::Recursive) {
            warn!("Failed to watch tasks dir (will retry): {e}");
        }
    } else {
        info!("Tasks directory doesn't exist yet, will watch parent");
        // Watch the parent (.claude) so we catch when tasks/ is created
        if let Some(parent) = tasks_dir.parent() {
            if parent.exists() {
                let _ = tasks_watcher.watch(parent, RecursiveMode::NonRecursive);
            }
        }
    }

    // Projects watcher (for metadata changes)
    let tx_projects = tx;
    let projects_watcher_result = notify::recommended_watcher(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                for path in &event.paths {
                    if path.extension().is_some_and(|e| e == "jsonl") {
                        let payload = serde_json::json!({ "type": "metadata-update" });
                        let _ = tx_projects.send(payload.to_string());
                    }
                }
            }
        },
    );

    let mut projects_watcher = match projects_watcher_result {
        Ok(w) => w,
        Err(e) => {
            error!("Failed to create projects watcher: {e}");
            return Some((tasks_watcher, notify::recommended_watcher(|_| {}).unwrap()));
        }
    };

    if projects_dir.exists() {
        if let Err(e) = projects_watcher.watch(projects_dir, RecursiveMode::Recursive) {
            warn!("Failed to watch projects dir: {e}");
        }
    }

    Some((tasks_watcher, projects_watcher))
}
