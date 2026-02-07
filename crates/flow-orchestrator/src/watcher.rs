//! File system watcher for syncing Claude Code tasks to features.

use crate::error::{OrchestratorError, Result};
use flow_core::{FlowEvent, Task};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Watches the Claude Code tasks directory for changes and broadcasts events.
pub struct TaskWatcher {
    /// Path to the tasks directory (~/.claude/tasks/).
    tasks_dir: PathBuf,
    /// Event broadcast sender.
    event_tx: broadcast::Sender<FlowEvent>,
    /// The file watcher (debounced).
    debouncer: Option<Debouncer<notify::RecommendedWatcher, FileIdMap>>,
}

impl TaskWatcher {
    /// Create a new task watcher for the given tasks directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory doesn't exist or watcher setup fails.
    pub fn new(tasks_dir: PathBuf, event_tx: broadcast::Sender<FlowEvent>) -> Result<Self> {
        info!(path = %tasks_dir.display(), "Creating task watcher");

        // Ensure the directory exists
        if !tasks_dir.exists() {
            warn!(path = %tasks_dir.display(), "Tasks directory does not exist, creating");
            std::fs::create_dir_all(&tasks_dir)?;
        }

        Ok(Self {
            tasks_dir,
            event_tx,
            debouncer: None,
        })
    }

    /// Start watching the tasks directory for changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the watcher fails to start.
    pub fn start(&mut self) -> Result<()> {
        info!(path = %self.tasks_dir.display(), "Starting task watcher");

        let event_tx = self.event_tx.clone();
        let tasks_dir = self.tasks_dir.clone();

        // Create a debounced watcher
        let mut debouncer = new_debouncer(
            Duration::from_millis(300),
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    for event in events {
                        if let Err(e) = Self::handle_event(&event.event, &tasks_dir, &event_tx) {
                            error!(error = %e, "Failed to handle file event");
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        error!(error = %error, "File watcher error");
                    }
                }
            },
        )
        .map_err(|e| OrchestratorError::Watcher(format!("failed to create watcher: {e}")))?;

        // Watch the tasks directory
        let watcher = debouncer.watcher();
        watcher
            .watch(&self.tasks_dir, RecursiveMode::Recursive)
            .map_err(|e| OrchestratorError::Watcher(format!("failed to watch directory: {e}")))?;

        self.debouncer = Some(debouncer);
        info!("Task watcher started successfully");

        Ok(())
    }

    /// Handle a file system event.
    fn handle_event(
        event: &Event,
        tasks_dir: &Path,
        event_tx: &broadcast::Sender<FlowEvent>,
    ) -> Result<()> {
        // Only process JSON files
        let path = if let Some(path) = event.paths.first() {
            path
        } else {
            return Ok(());
        };

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            return Ok(());
        }

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                debug!(path = %path.display(), "Task file created/modified");

                // Parse the task file
                match Self::parse_task_file(path) {
                    Ok(task) => {
                        // Extract session ID from path
                        let session_id = Self::extract_session_id(path, tasks_dir);

                        // Broadcast update event
                        let _ = event_tx.send(FlowEvent::TaskUpdate {
                            event: "modified".to_string(),
                            session_id,
                            file: path.file_name().and_then(|s| s.to_str()).map(String::from),
                        });

                        debug!(
                            task_id = %task.id,
                            subject = %task.subject,
                            "Task update broadcast"
                        );
                    }
                    Err(e) => {
                        warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to parse task file"
                        );
                    }
                }
            }
            EventKind::Remove(_) => {
                debug!(path = %path.display(), "Task file removed");

                let session_id = Self::extract_session_id(path, tasks_dir);

                let _ = event_tx.send(FlowEvent::TaskUpdate {
                    event: "deleted".to_string(),
                    session_id,
                    file: path.file_name().and_then(|s| s.to_str()).map(String::from),
                });
            }
            _ => {
                // Ignore other events
            }
        }

        Ok(())
    }

    /// Parse a task JSON file.
    fn parse_task_file(path: &Path) -> Result<Task> {
        let contents = std::fs::read_to_string(path)?;
        let task: Task = serde_json::from_str(&contents)?;
        Ok(task)
    }

    /// Extract session ID from a task file path.
    ///
    /// Path format: ~/.claude/tasks/{session-id}/{task-id}.json
    fn extract_session_id(path: &Path, tasks_dir: &Path) -> Option<String> {
        let relative = path.strip_prefix(tasks_dir).ok()?;
        let components: Vec<_> = relative.components().collect();

        if components.len() >= 2 {
            components[0].as_os_str().to_str().map(String::from)
        } else {
            None
        }
    }

    /// Stop watching the tasks directory.
    pub fn stop(&mut self) {
        if self.debouncer.is_some() {
            info!("Stopping task watcher");
            self.debouncer = None;
        }
    }
}

impl Drop for TaskWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_session_id() {
        let tasks_dir = PathBuf::from("/home/user/.claude/tasks");
        let task_path = PathBuf::from("/home/user/.claude/tasks/session-123/task-1.json");

        let session_id = TaskWatcher::extract_session_id(&task_path, &tasks_dir);
        assert_eq!(session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_parse_task_file() {
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let task_file = temp_dir.path().join("task.json");

        let task_json = r#"{
            "id": "1",
            "subject": "Test task",
            "description": "Test description",
            "activeForm": "Working on it",
            "status": "in_progress",
            "owner": "claude",
            "blocks": [],
            "blockedBy": []
        }"#;

        let mut file = std::fs::File::create(&task_file).unwrap();
        file.write_all(task_json.as_bytes()).unwrap();
        drop(file);

        let task = TaskWatcher::parse_task_file(&task_file).unwrap();
        assert_eq!(task.id, "1");
        assert_eq!(task.subject, "Test task");
    }
}
