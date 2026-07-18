use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use serde_json::json;
use time::Duration as TimeDuration;

use crate::auth::{
	consume_magic_link, create_magic_link, create_session, destroy_session, find_or_create_user_by_email,
	is_email_allowed, redeem_invite_link, SESSION_COOKIE,
};
use crate::email::send_magic_link_email;
use crate::error::AppResult;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct MagicLinkRequest {
	email: Option<String>,
	// Optional invite-link token (from an SMS invite's ?invite=… link). Lets a
	// brand-new person whose email was never directly allowlisted onboard
	// themselves, since a number-only invite never knew their email.
	invite: Option<String>,
}

pub async fn magic_link(State(state): State<AppState>, Json(body): Json<MagicLinkRequest>) -> AppResult<Response> {
	let Some(email) = body.email.filter(|e| e.contains('@')) else {
		return Ok((axum::http::StatusCode::BAD_REQUEST, Json(json!({ "error": "invalid email" }))).into_response());
	};

	// An unallowlisted email can still be let in by redeeming a valid invite
	// link — the inviter vouched for whoever holds it.
	let allowed = is_email_allowed(&state.pool, &email).await?
		|| match &body.invite {
			Some(token) => redeem_invite_link(&state.pool, token, &email).await?,
			None => false,
		};

	// Always respond the same way whether or not the email is allowed, so
	// the invite-only allowlist can't be probed from the sign-in form.
	if allowed {
		let token = create_magic_link(&state.pool, &email).await?;
		let link = format!("{}/auth/callback?token={token}", state.config.public_origin);
		send_magic_link_email(&state.config, &email, &link).await;
	}
	Ok(Json(json!({ "ok": true })).into_response())
}

#[derive(Deserialize)]
pub struct CallbackQuery {
	token: Option<String>,
}

pub async fn callback(
	State(state): State<AppState>,
	Query(query): Query<CallbackQuery>,
) -> AppResult<Response> {
	let email = match query.token {
		Some(token) => consume_magic_link(&state.pool, &token).await?,
		None => None,
	};
	let Some(email) = email else {
		return Ok(Redirect::to("/signin?error=expired").into_response());
	};

	let user = find_or_create_user_by_email(&state.pool, &email).await?;
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
	Ok((jar, Redirect::to(dest)).into_response())
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> AppResult<Response> {
	if let Some(cookie) = jar.get(SESSION_COOKIE) {
		if let Ok(session_id) = cookie.value().parse() {
			destroy_session(&state.pool, session_id).await?;
		}
	}
	let jar = jar.remove(Cookie::from(SESSION_COOKIE));
	Ok((jar, Json(json!({ "ok": true }))).into_response())
}
