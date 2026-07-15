use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::Json;
use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::json;

use crate::error::AppResult;
use crate::extractors::RequireUser;
use crate::repo::calendar;
use crate::state::AppState;
use crate::types::{Availability, CalendarDayEntryDto};

pub async fn get_days(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
) -> AppResult<Json<HashMap<NaiveDate, CalendarDayEntryDto>>> {
	let days = calendar::get_calendar_days(&state.pool, user.id).await?;
	Ok(Json(days))
}

#[derive(Deserialize)]
pub struct PatchDayBody {
	#[serde(default, deserialize_with = "deserialize_optional_field")]
	status: Option<Option<Availability>>,
	note: Option<String>,
}

// Distinguishes "status omitted" from "status: null" the same way the old
// `'status' in body` check did in TypeScript.
fn deserialize_optional_field<'de, D>(deserializer: D) -> Result<Option<Option<Availability>>, D::Error>
where
	D: serde::Deserializer<'de>,
{
	Ok(Some(Option::deserialize(deserializer)?))
}

pub async fn patch_day(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(date): Path<NaiveDate>,
	Json(body): Json<PatchDayBody>,
) -> AppResult<Json<serde_json::Value>> {
	if let Some(status) = body.status {
		calendar::set_day_status(&state.pool, user.id, date, status).await?;
	}
	if let Some(note) = body.note {
		calendar::set_day_note(&state.pool, user.id, date, &note).await?;
	}
	Ok(Json(json!({ "ok": true })))
}

pub async fn delete_day(
	State(state): State<AppState>,
	RequireUser(user): RequireUser,
	Path(date): Path<NaiveDate>,
) -> AppResult<Json<serde_json::Value>> {
	calendar::clear_day(&state.pool, user.id, date).await?;
	Ok(Json(json!({ "ok": true })))
}
