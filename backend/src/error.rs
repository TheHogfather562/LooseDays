use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
	#[error("sign in required")]
	Unauthorized,
	#[error("{0}")]
	BadRequest(String),
	#[error("{0}")]
	NotFound(String),
	#[error("{0}")]
	Conflict(String),
	#[error("{0}")]
	Forbidden(String),
	#[error(transparent)]
	Db(#[from] sqlx::Error),
	#[error(transparent)]
	Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, message) = match &self {
			AppError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
			AppError::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
			AppError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
			AppError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()),
			AppError::Forbidden(_) => (StatusCode::FORBIDDEN, self.to_string()),
			AppError::Db(e) => {
				tracing::error!("db error: {e:?}");
				(StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
			}
			AppError::Internal(e) => {
				tracing::error!("internal error: {e:?}");
				(StatusCode::INTERNAL_SERVER_ERROR, "internal error".to_string())
			}
		};
		(status, Json(json!({ "message": message }))).into_response()
	}
}

pub type AppResult<T> = Result<T, AppError>;
