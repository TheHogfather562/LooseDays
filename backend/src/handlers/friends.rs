use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::extractors::RequireUser;
use crate::repo::{access, friends};
use crate::state::AppState;
use crate::types::{AccessScope, Availability, ContactDto, FriendDto};
use std::collections::HashMap;

pub async fn list_friends(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<Vec<FriendDto>>> {
	Ok(Json(friends::get_friends(&state.pool, user.id).await?))
}

#[derive(Deserialize)]
pub struct AddFriendBody {
	#[serde(rename = "userId")]
	user_id: Option<Uuid>,
}

pub async fn add_friend(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Json(body): Json<AddFriendBody>,
) -> AppResult<Json<serde_json::Value>> {
	let Some(friend_id) = body.user_id else {
		return Err(AppError::BadRequest("userId required".into()));
	};
	friends::add_friend(&state.pool, user.id, friend_id).await?;
	Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct MatchContactsBody {
	entries: Option<Vec<ContactEntry>>,
}

#[derive(Deserialize)]
pub struct ContactEntry {
	name: String,
	phone: String,
}

pub async fn match_contacts(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Json(body): Json<MatchContactsBody>,
) -> AppResult<Json<Vec<ContactDto>>> {
	let entries: Vec<(String, String)> = body
		.entries
		.unwrap_or_default()
		.into_iter()
		.map(|e| (e.name, e.phone))
		.collect();
	let contacts = friends::match_contacts(
		&state.pool,
		user.id,
		&entries,
		&state.config.server_pepper,
		&state.config.default_phone_region,
	)
	.await?;
	Ok(Json(contacts))
}

#[derive(Deserialize)]
pub struct AccessRequestBody {
	scope: AccessScope,
	start: Option<chrono::NaiveDate>,
	end: Option<chrono::NaiveDate>,
}

pub async fn request_access(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(friend_id): Path<Uuid>,
	Json(body): Json<AccessRequestBody>,
) -> AppResult<Json<serde_json::Value>> {
	access::request_access(&state.pool, user.id, friend_id, body.scope, body.start, body.end).await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn friend_calendar(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(friend_id): Path<Uuid>,
) -> AppResult<Json<HashMap<chrono::NaiveDate, Availability>>> {
	Ok(Json(access::get_visible_friend_calendar(&state.pool, user.id, friend_id).await?))
}
