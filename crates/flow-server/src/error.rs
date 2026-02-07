use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};

/// Application error types
pub enum AppError {
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

impl From<flow_core::FlowError> for AppError {
    fn from(err: flow_core::FlowError) -> Self {
        Self::Internal(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
