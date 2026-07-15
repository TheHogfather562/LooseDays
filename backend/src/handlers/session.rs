use axum::Json;
use serde_json::json;

use crate::error::AppResult;
use crate::extractors::CurrentUser;

pub async fn get_session(CurrentUser(user): CurrentUser) -> AppResult<Json<serde_json::Value>> {
	Ok(Json(json!({
		"signedIn": user.is_some(),
		"onboarded": user.as_ref().is_some_and(|u| u.onboarded),
		"user": user.as_ref().map(|u| json!({
			"id": u.id,
			"displayName": u.display_name,
			"email": u.email,
			"phoneSet": u.phone_set,
		})),
	})))
}
