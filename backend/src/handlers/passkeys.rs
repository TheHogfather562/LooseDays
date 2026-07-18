use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use serde_json::json;
use time::Duration as TimeDuration;
use uuid::Uuid;
use webauthn_rs::prelude::{
	DiscoverableAuthentication, DiscoverableKey, Passkey, PasskeyRegistration, PublicKeyCredential,
	RegisterPublicKeyCredential,
};

use crate::auth::{create_session, get_user_by_id, SESSION_COOKIE};
use crate::error::{AppError, AppResult};
use crate::extractors::RequireUser;
use crate::repo::passkeys as repo;
use crate::state::AppState;

pub async fn register_start(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<serde_json::Value>> {
	let existing = repo::load_passkeys_for_user(&state.pool, user.id).await?;
	let exclude_credentials = (!existing.is_empty()).then(|| existing.iter().map(|p| p.cred_id().clone()).collect());

	let (ccr, reg_state) =
		state
			.webauthn
			.start_passkey_registration(user.id, &user.email, &user.display_name, exclude_credentials)?;
	let challenge_id = repo::store_challenge(&state.pool, "registration", Some(user.id), &reg_state).await?;

	Ok(Json(json!({ "challengeId": challenge_id, "publicKey": ccr.public_key })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterFinishBody {
	challenge_id: Uuid,
	credential: RegisterPublicKeyCredential,
	label: Option<String>,
}

pub async fn register_finish(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Json(body): Json<RegisterFinishBody>,
) -> AppResult<Json<serde_json::Value>> {
	let Some((challenge_user_id, reg_state)) =
		repo::take_challenge::<PasskeyRegistration>(&state.pool, body.challenge_id, "registration").await?
	else {
		return Err(AppError::BadRequest("registration challenge expired, try again".into()));
	};
	if challenge_user_id != Some(user.id) {
		return Err(AppError::Forbidden("challenge does not belong to this session".into()));
	}

	let passkey = state.webauthn.finish_passkey_registration(&body.credential, &reg_state)?;
	let label = body.label.filter(|l| !l.trim().is_empty()).unwrap_or_else(|| "Passkey".to_string());
	repo::insert_credential(&state.pool, user.id, passkey.cred_id().as_ref(), &passkey, &label).await?;

	Ok(Json(json!({ "ok": true })))
}

pub async fn list(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<Vec<crate::types::PasskeyDto>>> {
	Ok(Json(repo::list_credentials(&state.pool, user.id).await?))
}

pub async fn remove(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
	if !repo::delete_credential(&state.pool, user.id, id).await? {
		return Err(AppError::NotFound("passkey not found".into()));
	}
	Ok(Json(json!({ "ok": true })))
}

/// Starts a "discoverable" (usernameless) sign-in: the browser's own
/// credential picker identifies which passkey to use, so no email/username
/// needs to be collected first.
pub async fn auth_start(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
	let (rcr, auth_state) = state.webauthn.start_discoverable_authentication()?;
	let challenge_id = repo::store_challenge(&state.pool, "authentication", None, &auth_state).await?;
	Ok(Json(json!({ "challengeId": challenge_id, "publicKey": rcr.public_key })))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthFinishBody {
	challenge_id: Uuid,
	credential: PublicKeyCredential,
}

pub async fn auth_finish(State(state): State<AppState>, Json(body): Json<AuthFinishBody>) -> AppResult<Response> {
	let Some((_, auth_state)) =
		repo::take_challenge::<DiscoverableAuthentication>(&state.pool, body.challenge_id, "authentication").await?
	else {
		return Err(AppError::BadRequest("sign-in challenge expired, try again".into()));
	};

	let (user_id, cred_id) = state.webauthn.identify_discoverable_authentication(&body.credential)?;
	let stored = repo::load_passkeys_for_user(&state.pool, user_id).await?;
	let discoverable: Vec<DiscoverableKey> = stored.iter().map(DiscoverableKey::from).collect();

	let auth_result = state
		.webauthn
		.finish_discoverable_authentication(&body.credential, auth_state, &discoverable)?;

	if let Some(mut passkey) = stored.into_iter().find(|p: &Passkey| p.cred_id().as_ref() == cred_id) {
		passkey.update_credential(&auth_result);
		repo::update_after_auth(&state.pool, user_id, cred_id, &passkey).await?;
	}

	let Some(user) = get_user_by_id(&state.pool, user_id).await? else {
		return Err(AppError::Unauthorized);
	};
	let (session_id, expires_at) = create_session(&state.pool, user.id).await?;

	let max_age = expires_at - chrono::Utc::now();
	let cookie = Cookie::build((SESSION_COOKIE, session_id.to_string()))
		.path("/")
		.http_only(true)
		.secure(state.config.cookie_secure)
		.same_site(SameSite::Lax)
		.max_age(TimeDuration::seconds(max_age.num_seconds().max(0)))
		.build();
	let jar = CookieJar::new().add(cookie);
	let dest = if user.onboarded { "/calendar" } else { "/onboarding" };
	Ok((jar, Json(json!({ "ok": true, "redirect": dest }))).into_response())
}
