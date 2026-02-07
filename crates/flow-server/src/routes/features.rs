use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{delete, get, post},
    Router,
};
use flow_core::{CreateFeatureInput, DependencyRef, Feature, FeatureGraphNode, FeatureStats};
use flow_db::FeatureStore;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct BulkCreateRequest {
    features: Vec<CreateFeatureInput>,
}

/// Build the feature routes sub-router
pub fn feature_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_features).post(create_feature))
        .route("/bulk", post(bulk_create))
        .route("/stats", get(get_stats))
        .route("/ready", get(get_ready))
        .route("/blocked", get(get_blocked))
        .route("/graph", get(get_graph))
        .route("/:id", get(get_feature_by_id))
        .route("/:id/claim", post(claim_feature))
        .route("/:id/pass", post(mark_passing))
        .route("/:id/fail", post(mark_failing))
        .route("/:id/skip", post(skip_feature))
        .route("/:id/dependencies", post(add_dependency))
        .route("/:id/dependencies/:dep_id", delete(remove_dependency))
}

/// GET /api/features — List all features
async fn list_features(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Feature>>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let features = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::get_all(&conn)?
    };
    Ok(Json(features))
}

/// GET /api/features/:id — Get feature by ID
async fn get_feature_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<Feature>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    let feature = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::get_by_id(&conn, id_num)?
            .ok_or_else(|| AppError::NotFound("Feature not found".into()))?
    };
    Ok(Json(feature))
}

/// GET /api/features/stats — Get feature statistics
async fn get_stats(State(state): State<Arc<AppState>>) -> AppResult<Json<FeatureStats>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let feature_stats = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::get_stats(&conn)?
    };
    Ok(Json(feature_stats))
}

/// GET /api/features/ready — Get ready features
async fn get_ready(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Feature>>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let features = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::get_ready(&conn)?
    };
    Ok(Json(features))
}

/// GET /api/features/blocked — Get blocked features
async fn get_blocked(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<Feature>>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let features = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::get_blocked(&conn)?
    };
    Ok(Json(features))
}

/// GET /api/features/graph — Get dependency graph
async fn get_graph(State(state): State<Arc<AppState>>) -> AppResult<Json<Vec<FeatureGraphNode>>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let features = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::get_all(&conn)?
    };

    // Build reverse dependency map (dependents)
    let mut dependents_map: std::collections::HashMap<i64, Vec<i64>> =
        std::collections::HashMap::new();
    for feature in &features {
        for &dep_id in &feature.dependencies {
            dependents_map.entry(dep_id).or_default().push(feature.id);
        }
    }

    // Clone features for checking blocked status
    let features_for_check = features.clone();

    // Build graph from features
    let graph: Vec<FeatureGraphNode> = features
        .into_iter()
        .map(|f| {
            let is_blocked = !f.dependencies.is_empty()
                && f.dependencies.iter().any(|dep_id| {
                    features_for_check
                        .iter()
                        .any(|other| other.id == *dep_id && !other.passes)
                });

            FeatureGraphNode {
                id: f.id,
                name: f.name,
                category: f.category,
                priority: f.priority,
                passes: f.passes,
                in_progress: f.in_progress,
                blocked: is_blocked,
                dependencies: f.dependencies.clone(),
                dependents: dependents_map.get(&f.id).cloned().unwrap_or_default(),
            }
        })
        .collect();

    Ok(Json(graph))
}

/// POST /api/features — Create a new feature
async fn create_feature(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateFeatureInput>,
) -> AppResult<Json<Feature>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let feature = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::create(&conn, &input)?
    };
    Ok(Json(feature))
}

/// POST /api/features/bulk — Bulk create features
async fn bulk_create(
    State(state): State<Arc<AppState>>,
    Json(request): Json<BulkCreateRequest>,
) -> AppResult<Json<Vec<Feature>>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let features = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::create_bulk(&conn, &request.features)?
    };
    Ok(Json(features))
}

/// POST /api/features/:id/claim — Claim a feature
async fn claim_feature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<Feature>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    let feature = {
        let conn = db.writer().lock().unwrap();
        FeatureStore::claim_and_get(&conn, id_num)?
    };
    Ok(Json(feature))
}

/// POST /api/features/:id/pass — Mark feature as passing
async fn mark_passing(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    let conn = db.writer().lock().unwrap();
    FeatureStore::mark_passing(&conn, id_num)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/features/:id/fail — Mark feature as failing
async fn mark_failing(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    let conn = db.writer().lock().unwrap();
    FeatureStore::mark_failing(&conn, id_num)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/features/:id/skip — Skip a feature
async fn skip_feature(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    let conn = db.writer().lock().unwrap();
    FeatureStore::skip(&conn, id_num)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/features/:id/dependencies — Add a dependency
async fn add_dependency(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(dep): Json<DependencyRef>,
) -> AppResult<Json<serde_json::Value>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    // Extract the dependency ID from `DependencyRef`
    let DependencyRef::Id(dep_id) = dep else {
        return Err(AppError::BadRequest("Invalid dependency reference".into()));
    };

    let conn = db.writer().lock().unwrap();
    FeatureStore::add_dependency(&conn, id_num, dep_id)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// DELETE `/api/features/:id/dependencies/:dep_id` — Remove a dependency
async fn remove_dependency(
    State(state): State<Arc<AppState>>,
    Path((id, dep_id)): Path<(String, String)>,
) -> AppResult<Json<serde_json::Value>> {
    let db = state
        .db
        .as_ref()
        .ok_or_else(|| AppError::Internal("Database not configured".into()))?;

    let id_num: i64 = id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid feature ID".into()))?;

    let dep_id_num: i64 = dep_id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid dependency ID".into()))?;

    let conn = db.writer().lock().unwrap();
    FeatureStore::remove_dependency(&conn, id_num, dep_id_num)?;
    Ok(Json(serde_json::json!({ "success": true })))
}
