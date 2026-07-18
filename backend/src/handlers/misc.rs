use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::auth::{
	complete_onboarding, create_invite_link, email_has_account, invite_email, set_user_phone,
};
use crate::email::send_invite_email;
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

	// Allowlist the email either way (idempotent). Only send an invite email
	// to genuinely new people — someone who already has an account doesn't need
	// to be told they've been "invited".
	let already_user = email_has_account(&state.pool, &email).await?;
	invite_email(&state.pool, &email, user.id).await?;
	if !already_user {
		let signin_url = format!("{}/signin", state.config.public_origin);
		send_invite_email(&state.config, &email, &user.display_name, &signin_url).await;
	}
	Ok(Json(json!({ "ok": true })))
}

/// Mints a shareable invite-link token the caller can drop into the SMS/WhatsApp
/// invites they hand off. Redeeming it at sign-in allowlists the invitee's own
/// email — the missing piece for number-only invites, where we never learn the
/// email up front.
pub async fn create_invite(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<serde_json::Value>> {
	let token = create_invite_link(&state.pool, user.id).await?;
	Ok(Json(json!({ "token": token })))
}
