use crate::{
    error::{AppError, AppResult},
    helpers::format_system_time,
    state::{get_metadata, AppState},
};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use flow_core::{SessionListItem, Task};
use serde::Deserialize;
use std::{fs, sync::Arc, time::SystemTime};

#[derive(Debug, Deserialize)]
pub struct SessionQuery {
    limit: Option<String>,
}

/// GET /api/sessions — List all sessions with task summaries
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SessionQuery>,
) -> AppResult<Json<Vec<SessionListItem>>> {
    let limit_str = query.limit.unwrap_or_else(|| "20".to_string());
    let limit: Option<usize> = if limit_str == "all" {
        None
    } else {
        limit_str.parse().ok()
    };

    let metadata = get_metadata(&state).await;
    let mut sessions = Vec::new();

    if state.tasks_dir.exists() {
        let Ok(entries) = fs::read_dir(&state.tasks_dir) else {
            return Err(AppError::Internal("Failed to read tasks directory".into()));
        };

        for entry in entries.flatten() {
            if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                continue;
            }

            let session_id = entry.file_name().to_string_lossy().to_string();
            let session_path = entry.path();

            let Ok(dir_stat) = fs::metadata(&session_path) else {
                continue;
            };

            let Ok(task_entries) = fs::read_dir(&session_path) else {
                continue;
            };

            let mut completed = 0usize;
            let mut in_progress = 0usize;
            let mut pending = 0usize;
            let mut task_count = 0usize;
            let mut newest_mtime: Option<SystemTime> = None;

            for task_entry in task_entries.flatten() {
                let fname = task_entry.file_name();
                if !fname.to_string_lossy().ends_with(".json") {
                    continue;
                }

                task_count += 1;
                let task_path = task_entry.path();

                if let Ok(content) = fs::read_to_string(&task_path) {
                    if let Ok(task) = serde_json::from_str::<Task>(&content) {
                        match task.status.as_str() {
                            "completed" => completed += 1,
                            "in_progress" => in_progress += 1,
                            _ => pending += 1,
                        }
                    }
                }

                if let Ok(task_stat) = fs::metadata(&task_path) {
                    if let Ok(mtime) = task_stat.modified() {
                        newest_mtime = Some(newest_mtime.map_or(mtime, |prev| prev.max(mtime)));
                    }
                }
            }

            let meta = metadata.get(&session_id);
            let modified_at = newest_mtime
                .or_else(|| dir_stat.modified().ok())
                .map(|t| {
                    let duration = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
                    format_system_time(duration)
                })
                .unwrap_or_default();

            sessions.push(SessionListItem {
                id: session_id.clone(),
                name: meta.and_then(flow_core::SessionMeta::display_name),
                slug: meta.and_then(|m| m.slug.clone()),
                project: meta.and_then(|m| m.project_path.clone()),
                description: meta.and_then(|m| m.description.clone()),
                git_branch: meta.and_then(|m| m.git_branch.clone()),
                task_count,
                completed,
                in_progress,
                pending,
                created_at: meta.and_then(|m| m.created.clone()),
                modified_at,
            });
        }
    }

    // Sort by most recently modified
    sessions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    // Apply limit
    if let Some(limit) = limit {
        sessions.truncate(limit);
    }

    Ok(Json(sessions))
}

/// GET `/api/sessions/:session_id` — Get tasks for a session
pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> AppResult<Json<Vec<Task>>> {
    let session_path = state.tasks_dir.join(&session_id);

    if !session_path.exists() {
        return Err(AppError::NotFound("Session not found".into()));
    }

    let Ok(entries) = fs::read_dir(&session_path) else {
        return Err(AppError::Internal(
            "Failed to read session directory".into(),
        ));
    };

    let mut tasks: Vec<Task> = entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with(".json"))
        .filter_map(|e| {
            fs::read_to_string(e.path())
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        })
        .collect();

    // Sort by numeric ID
    tasks.sort_by(|a, b| {
        a.id.parse::<u64>()
            .unwrap_or(0)
            .cmp(&b.id.parse::<u64>().unwrap_or(0))
    });

    Ok(Json(tasks))
}
