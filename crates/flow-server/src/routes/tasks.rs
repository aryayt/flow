use crate::{error::{AppError, AppResult}, state::{get_metadata, AppState}};
use axum::{
    extract::{Path, State},
    response::Json,
};
use flow_core::{Task, TaskWithSession};
use serde::Deserialize;
use std::{fs, sync::Arc};

#[derive(Debug, Deserialize)]
pub struct NoteRequest {
    note: String,
}

/// GET /api/tasks/all — All tasks across all sessions
pub async fn get_all_tasks(
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<Vec<TaskWithSession>>> {
    if !state.tasks_dir.exists() {
        return Ok(Json(vec![]));
    }

    let metadata = get_metadata(&state).await;

    let Ok(session_dirs) = fs::read_dir(&state.tasks_dir) else {
        return Ok(Json(vec![]));
    };

    let mut all_tasks = Vec::new();

    for session_entry in session_dirs.flatten() {
        if !session_entry
            .file_type()
            .map(|ft| ft.is_dir())
            .unwrap_or(false)
        {
            continue;
        }

        let session_id = session_entry.file_name().to_string_lossy().to_string();
        let meta = metadata.get(&session_id);

        let Ok(task_files) = fs::read_dir(session_entry.path()) else {
            continue;
        };

        for task_entry in task_files.flatten() {
            if !task_entry.file_name().to_string_lossy().ends_with(".json") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(task_entry.path()) {
                if let Ok(task) = serde_json::from_str::<Task>(&content) {
                    all_tasks.push(TaskWithSession {
                        task,
                        session_id: session_id.clone(),
                        session_name: meta.and_then(flow_core::SessionMeta::display_name),
                        project: meta.and_then(|m| m.project_path.clone()),
                    });
                }
            }
        }
    }

    Ok(Json(all_tasks))
}

/// POST `/api/tasks/:session_id/:task_id/note` — Add note to a task
pub async fn add_note(
    State(state): State<Arc<AppState>>,
    Path((session_id, task_id)): Path<(String, String)>,
    Json(body): Json<NoteRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let note = body.note.trim();
    if note.is_empty() {
        return Err(AppError::BadRequest("Note cannot be empty".into()));
    }

    let task_path = state.tasks_dir.join(&session_id).join(format!("{task_id}.json"));

    if !task_path.exists() {
        return Err(AppError::NotFound("Task not found".into()));
    }

    let content = fs::read_to_string(&task_path)
        .map_err(|e| AppError::Internal(format!("Failed to read task: {e}")))?;

    let mut task: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| AppError::Internal(format!("Failed to parse task: {e}")))?;

    // Append note to description
    let current_desc = task
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let note_block = format!("\n\n---\n\n#### [Note added by user]\n\n{note}");
    task["description"] = serde_json::Value::String(format!("{current_desc}{note_block}"));

    let output = serde_json::to_string_pretty(&task)
        .map_err(|e| AppError::Internal(format!("Failed to serialize task: {e}")))?;
    fs::write(&task_path, output)
        .map_err(|e| AppError::Internal(format!("Failed to write task: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true, "task": task })))
}

/// DELETE `/api/tasks/:session_id/:task_id` — Delete a task
pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    Path((session_id, task_id)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let session_path = state.tasks_dir.join(&session_id);
    let task_path = session_path.join(format!("{task_id}.json"));

    if !task_path.exists() {
        return Err(AppError::NotFound("Task not found".into()));
    }

    // Check if this task blocks other tasks
    if let Ok(entries) = fs::read_dir(&session_path) {
        for entry in entries.flatten() {
            if !entry.file_name().to_string_lossy().ends_with(".json") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Ok(other_task) = serde_json::from_str::<Task>(&content) {
                    if other_task.blocked_by.contains(&task_id) {
                        return Err(AppError::BadRequest(format!(
                            "Cannot delete task that blocks other tasks (blocked: {})",
                            other_task.id
                        )));
                    }
                }
            }
        }
    }

    fs::remove_file(&task_path)
        .map_err(|e| AppError::Internal(format!("Failed to delete task: {e}")))?;

    Ok(Json(serde_json::json!({ "success": true, "taskId": task_id })))
}
