use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::error::AppResult;
use crate::extractors::RequireUser;
use crate::repo::access;
use crate::state::AppState;
use crate::types::{DetailLevel, FriendAccessGrantDto, IncomingRequestDto, StandingAccessGrantDto};

pub async fn incoming(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<Vec<IncomingRequestDto>>> {
	Ok(Json(access::get_incoming_requests(&state.pool, user.id).await?))
}

#[derive(Deserialize)]
pub struct ApproveBody {
	#[serde(rename = "detailLevel")]
	detail_level: DetailLevel,
}

pub async fn approve(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(request_id): Path<Uuid>,
	Json(body): Json<ApproveBody>,
) -> AppResult<Json<serde_json::Value>> {
	access::approve_request(&state.pool, request_id, user.id, body.detail_level).await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn deny(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(request_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
	access::deny_request(&state.pool, request_id, user.id).await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn mine(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<HashMap<Uuid, FriendAccessGrantDto>>> {
	Ok(Json(access::get_my_access(&state.pool, user.id).await?))
}

pub async fn outgoing(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<HashMap<Uuid, bool>>> {
	Ok(Json(access::get_outgoing_pending(&state.pool, user.id).await?))
}

pub async fn cancel_outgoing(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(owner_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
	access::cancel_outgoing(&state.pool, user.id, owner_id).await?;
	Ok(Json(json!({ "ok": true })))
}

pub async fn standing(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<Vec<StandingAccessGrantDto>>> {
	Ok(Json(access::get_standing_access(&state.pool, user.id).await?))
}

pub async fn revoke(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(request_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
	access::revoke_access(&state.pool, request_id, user.id).await?;
	Ok(Json(json!({ "ok": true })))
}
