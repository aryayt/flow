use flow_core::SessionMeta;
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, RwLock};

/// Application state shared across all handlers
pub struct AppState {
    pub tasks_dir: PathBuf,
    pub projects_dir: PathBuf,
    pub tx: broadcast::Sender<String>,
    pub metadata_cache: RwLock<MetadataCache>,
    pub db: Option<Arc<flow_db::Database>>,
}

/// Cache for session metadata with time-based invalidation
pub struct MetadataCache {
    pub data: HashMap<String, SessionMeta>,
    pub last_refresh: Instant,
}

impl Default for MetadataCache {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            last_refresh: Instant::now()
                .checked_sub(Duration::from_secs(60))
                .unwrap_or_else(Instant::now),
        }
    }

    pub fn is_stale(&self) -> bool {
        self.last_refresh.elapsed() > Duration::from_secs(10)
    }
}

/// Refresh metadata cache if stale, return current data
pub async fn get_metadata(state: &AppState) -> HashMap<String, SessionMeta> {
    {
        let cache = state.metadata_cache.read().await;
        if !cache.is_stale() {
            return cache.data.clone();
        }
    }

    let mut cache = state.metadata_cache.write().await;
    // Double-check after acquiring write lock
    if !cache.is_stale() {
        return cache.data.clone();
    }

    cache.data = load_session_metadata(&state.projects_dir);
    cache.last_refresh = Instant::now();
    cache.data.clone()
}

/// Scan all project directories to build session metadata cache
pub fn load_session_metadata(projects_dir: &std::path::Path) -> HashMap<String, SessionMeta> {
    let mut metadata = HashMap::new();

    let Ok(project_dirs) = fs::read_dir(projects_dir) else {
        return metadata;
    };

    for entry in project_dirs.flatten() {
        if !entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            continue;
        }

        let project_path = entry.path();

        // Find all .jsonl files (session logs)
        let Ok(files) = fs::read_dir(&project_path) else {
            continue;
        };

        for file_entry in files.flatten() {
            let file_name = file_entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.ends_with(".jsonl") {
                let session_id = file_name_str.trim_end_matches(".jsonl").to_string();
                let jsonl_path = file_entry.path();
                let session_info = read_session_info_from_jsonl(&jsonl_path);

                metadata.insert(session_id, session_info);
            }
        }

        // Also check sessions-index.json
        let index_path = project_path.join("sessions-index.json");
        if index_path.exists() {
            if let Ok(content) = fs::read_to_string(&index_path) {
                if let Ok(index_data) = serde_json::from_str::<Value>(&content) {
                    if let Some(entries) = index_data.get("entries").and_then(|v| v.as_array()) {
                        for entry in entries {
                            if let Some(sid) = entry.get("sessionId").and_then(|v| v.as_str()) {
                                if let Some(meta) = metadata.get_mut(sid) {
                                    if let Some(desc) =
                                        entry.get("description").and_then(|v| v.as_str())
                                    {
                                        meta.description = Some(desc.to_string());
                                    }
                                    if let Some(branch) =
                                        entry.get("gitBranch").and_then(|v| v.as_str())
                                    {
                                        meta.git_branch = Some(branch.to_string());
                                    }
                                    if let Some(created) =
                                        entry.get("created").and_then(|v| v.as_str())
                                    {
                                        meta.created = Some(created.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    metadata
}

/// Read customTitle, slug, and projectPath from first 64KB of a JSONL file
pub fn read_session_info_from_jsonl(path: &std::path::Path) -> SessionMeta {
    let mut meta = SessionMeta::default();

    let Ok(mut file) = File::open(path) else {
        return meta;
    };

    let mut buffer = vec![0u8; 65536];
    let bytes_read = file.read(&mut buffer).unwrap_or(0);
    buffer.truncate(bytes_read);

    let content = String::from_utf8_lossy(&buffer);

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let Ok(data) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        // Check for custom-title entry (from /rename command)
        if data.get("type").and_then(|v| v.as_str()) == Some("custom-title") {
            if let Some(title) = data.get("customTitle").and_then(|v| v.as_str()) {
                meta.custom_title = Some(title.to_string());
            }
        }

        // Check for slug
        if meta.slug.is_none() {
            if let Some(slug) = data.get("slug").and_then(|v| v.as_str()) {
                meta.slug = Some(slug.to_string());
            }
        }

        // Extract project path from cwd field
        if meta.project_path.is_none() {
            if let Some(cwd) = data.get("cwd").and_then(|v| v.as_str()) {
                meta.project_path = Some(cwd.to_string());
            }
        }

        // Stop early if we found all three
        if meta.custom_title.is_some() && meta.slug.is_some() && meta.project_path.is_some() {
            break;
        }
    }

    meta
}
