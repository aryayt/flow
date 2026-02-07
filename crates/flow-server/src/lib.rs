//! High-performance web server for Agent Flow task monitoring.
//!
//! Built on [axum](https://docs.rs/axum) with real-time updates via
//! Server-Sent Events (SSE) and WebSocket connections. Watches the
//! filesystem for task changes and broadcasts updates to all connected clients.
//!
//! # Features
//!
//! - REST API for sessions, tasks, and features
//! - SSE and WebSocket endpoints for live updates
//! - File watcher using the `notify` crate
//! - Optional `SQLite` database for feature management
//! - Static file serving for the web UI

pub mod error;
pub mod helpers;
pub mod routes;
pub mod sse;
pub mod state;
pub mod watcher;
pub mod ws;

pub use error::{AppError, AppResult};
pub use state::{AppState, MetadataCache};

use axum::{
    routing::{delete, get, post},
    Router,
};
use flow_core::AgentConfig;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower_http::services::ServeDir;
use tracing::info;

/// Build the axum router with all routes
pub fn build_router(state: Arc<AppState>) -> Router {
    let mut app = Router::new()
        .route("/api/sessions", get(routes::sessions::list_sessions))
        .route(
            "/api/sessions/:session_id",
            get(routes::sessions::get_session),
        )
        .route("/api/tasks/all", get(routes::tasks::get_all_tasks))
        .route(
            "/api/tasks/:session_id/:task_id/note",
            post(routes::tasks::add_note),
        )
        .route(
            "/api/tasks/:session_id/:task_id",
            delete(routes::tasks::delete_task),
        )
        .route("/api/events", get(sse::sse_handler))
        .route("/api/ws", get(ws::ws_handler))
        .route("/api/theme", get(routes::theme::get_theme))
        .route("/api/theme", post(routes::theme::set_theme));

    // Add feature routes if database is available
    if state.db.is_some() {
        app = app.nest("/api/features", routes::features::feature_routes());
    }

    app.with_state(state)
}

/// Run the axum server with the given configuration
#[allow(clippy::cognitive_complexity)]
pub async fn run_server(config: AgentConfig) -> flow_core::Result<()> {
    let tasks_dir = config.tasks_dir();
    let projects_dir = config.projects_dir();

    info!("Tasks directory: {}", tasks_dir.display());
    info!("Projects directory: {}", projects_dir.display());

    // Determine public directory
    let public_dir = config.public_dir.as_ref().map_or_else(
        || {
            // Default to ../public relative to binary, or ./public as fallback
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
                .map_or_else(
                    || std::path::PathBuf::from("public"),
                    |exe| {
                        let candidate = exe.join("..").join("public");
                        if candidate.exists() {
                            candidate
                        } else {
                            std::path::PathBuf::from("public")
                        }
                    },
                )
        },
        std::clone::Clone::clone,
    );

    info!("Public directory: {}", public_dir.display());

    // Create broadcast channel for SSE/WS (buffer 256 messages)
    let (tx, _) = broadcast::channel::<String>(256);

    // Database is optional - can be added later for feature management
    let db = None;

    let state = Arc::new(AppState {
        tasks_dir: tasks_dir.clone(),
        projects_dir: projects_dir.clone(),
        tx: tx.clone(),
        metadata_cache: RwLock::new(MetadataCache::new()),
        db,
    });

    // Set up file watchers (keep handles alive)
    let _watchers = watcher::setup_file_watcher(&tasks_dir, &projects_dir, tx);

    // Build router with fallback to serve static files
    let app = build_router(state.clone()).fallback_service(ServeDir::new(&public_dir));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Server running at http://localhost:{}", config.port);

    // Open browser if requested (cross-platform)
    if config.open_browser {
        let url = format!("http://localhost:{}", config.port);
        tokio::spawn(async move {
            let _ = open_browser(&url).await;
        });
    }

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(flow_core::FlowError::Io)?;

    axum::serve(listener, app)
        .await
        .map_err(|e| flow_core::FlowError::Io(std::io::Error::other(e)))?;

    Ok(())
}

/// Open a URL in the default browser (cross-platform).
async fn open_browser(url: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        tokio::process::Command::new("open")
            .arg(url)
            .status()
            .await?;
    }
    #[cfg(target_os = "linux")]
    {
        tokio::process::Command::new("xdg-open")
            .arg(url)
            .status()
            .await?;
    }
    #[cfg(target_os = "windows")]
    {
        tokio::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
            .await?;
    }
    Ok(())
}
