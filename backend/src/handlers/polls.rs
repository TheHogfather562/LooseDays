use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::extractors::RequireUser;
use crate::repo::polls;
use crate::state::AppState;
use crate::types::{Availability, CreatePollInput, PollDto};

pub async fn list(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<Vec<PollDto>>> {
	Ok(Json(polls::list_polls_for_user(&state.pool, user.id).await?))
}

pub async fn create(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Json(input): Json<CreatePollInput>,
) -> AppResult<Json<PollDto>> {
	let poll = polls::create_poll(
		&state.pool,
		user.id,
		input,
		&state.config.server_pepper,
		&state.config.default_phone_region,
	)
	.await?;
	Ok(Json(poll))
}

pub async fn get(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(poll_id): Path<Uuid>,
) -> AppResult<Json<PollDto>> {
	let poll = polls::get_poll_for_user(&state.pool, poll_id, user.id).await?;
	poll.map(Json).ok_or_else(|| AppError::NotFound("Poll not found".into()))
}

#[derive(Deserialize)]
pub struct RespondBody {
	responses: HashMap<chrono::NaiveDate, Availability>,
}

pub async fn respond(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(poll_id): Path<Uuid>,
	Json(body): Json<RespondBody>,
) -> AppResult<Json<serde_json::Value>> {
	let ok = polls::submit_response(&state.pool, poll_id, user.id, &body.responses).await?;
	if !ok {
		return Err(AppError::Forbidden("Not an invitee on this poll".into()));
	}
	Ok(Json(json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct FinalizeBody {
	start: chrono::NaiveDate,
	end: chrono::NaiveDate,
}

pub async fn finalize(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(poll_id): Path<Uuid>,
	Json(body): Json<FinalizeBody>,
) -> AppResult<Json<PollDto>> {
	let ok = polls::finalize_poll(&state.pool, poll_id, user.id, body.start, body.end).await?;
	if !ok {
		return Err(AppError::Forbidden("Only the poll's creator can finalize it".into()));
	}
	let poll = polls::get_poll_for_user(&state.pool, poll_id, user.id).await?;
	poll.map(Json).ok_or_else(|| AppError::NotFound("Poll not found".into()))
}

pub async fn reopen(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(poll_id): Path<Uuid>,
) -> AppResult<Json<PollDto>> {
	let ok = polls::reopen_poll(&state.pool, poll_id, user.id).await?;
	if !ok {
		return Err(AppError::Forbidden("Only the poll's creator can reopen it".into()));
	}
	let poll = polls::get_poll_for_user(&state.pool, poll_id, user.id).await?;
	poll.map(Json).ok_or_else(|| AppError::NotFound("Poll not found".into()))
}

pub async fn public_get(
	State(state): State<AppState>,
	Path(token): Path<String>,
) -> AppResult<Json<PollDto>> {
	let poll = polls::get_poll_by_token(&state.pool, &token).await?;
	poll.map(Json).ok_or_else(|| AppError::NotFound("Poll not found".into()))
}

pub async fn public_respond(
	State(state): State<AppState>,
	Path(token): Path<String>,
	Json(body): Json<RespondBody>,
) -> AppResult<Json<serde_json::Value>> {
	let ok = polls::submit_response_by_token(&state.pool, &token, &body.responses).await?;
	if !ok {
		return Err(AppError::NotFound("Poll not found".into()));
	}
	Ok(Json(json!({ "ok": true })))
}
