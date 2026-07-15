use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::auth::{complete_onboarding, invite_email, set_user_phone};
use crate::error::{AppError, AppResult};
use crate::extractors::RequireUser;
use crate::state::AppState;

pub async fn onboarding_complete(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<serde_json::Value>> {
	complete_onboarding(&state.pool, user.id).await?;
	Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct PhoneBody {
	phone: Option<String>,
}

pub async fn set_phone(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Json(body): Json<PhoneBody>,
) -> AppResult<Json<serde_json::Value>> {
	let Some(phone) = body.phone.filter(|p| !p.trim().is_empty()) else {
		return Err(AppError::BadRequest("phone required".into()));
	};
	set_user_phone(
		&state.pool,
		user.id,
		phone.trim(),
		&state.config.server_pepper,
		&state.config.default_phone_region,
	)
	.await?;
	Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct InviteBody {
	email: Option<String>,
}

pub async fn invite(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Json(body): Json<InviteBody>,
) -> AppResult<Json<serde_json::Value>> {
	let Some(email) = body.email.filter(|e| e.contains('@')) else {
		return Err(AppError::BadRequest("invalid email".into()));
	};
	invite_email(&state.pool, &email, user.id).await?;
	Ok(Json(json!({ "ok": true })))
}
