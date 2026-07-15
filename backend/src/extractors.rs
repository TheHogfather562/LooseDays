use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::auth::{get_user_by_session, SESSION_COOKIE};
use crate::error::AppError;
use crate::state::AppState;
use crate::types::SessionUserDto;

/// The signed-in user, if any — mirrors hooks.server.ts resolving
/// `event.locals.user` from the session cookie on every request.
pub struct CurrentUser(pub Option<SessionUserDto>);

#[async_trait::async_trait]
impl FromRequestParts<AppState> for CurrentUser {
	type Rejection = AppError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
		let jar = CookieJar::from_headers(&parts.headers);
		let Some(cookie) = jar.get(SESSION_COOKIE) else {
			return Ok(CurrentUser(None));
		};
		let Ok(session_id) = Uuid::parse_str(cookie.value()) else {
			return Ok(CurrentUser(None));
		};
		let user = get_user_by_session(&state.pool, session_id).await?;
		Ok(CurrentUser(user))
	}
}

/// Same as `CurrentUser` but rejects with 401 if nobody's signed in —
/// equivalent to the old `requireUser(event)` helper.
pub struct RequireUser(pub SessionUserDto);

#[async_trait::async_trait]
impl FromRequestParts<AppState> for RequireUser {
	type Rejection = AppError;

	async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
		let CurrentUser(user) = CurrentUser::from_request_parts(parts, state).await?;
		user.map(RequireUser).ok_or(AppError::Unauthorized)
	}
}
