use std::collections::HashMap;

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::types::{Availability, CalendarDayEntryDto};

#[derive(sqlx::FromRow)]
struct DayRow {
	date: NaiveDate,
	availability: Option<String>,
	note: String,
}

pub async fn get_calendar_days(
	pool: &PgPool,
	user_id: Uuid,
) -> AppResult<HashMap<NaiveDate, CalendarDayEntryDto>> {
	let rows = sqlx::query_as::<_, DayRow>(
		"SELECT date, availability, note FROM calendar_days WHERE user_id = $1",
	)
	.bind(user_id)
	.fetch_all(pool)
	.await?;

	let mut out = HashMap::new();
	for r in rows {
		if r.availability.is_some() || !r.note.is_empty() {
			let status = r.availability.map(|a| Availability::try_from(a.as_str())).transpose()?;
			out.insert(r.date, CalendarDayEntryDto { status, note: r.note });
		}
	}
	Ok(out)
}

async fn upsert(
	pool: &PgPool,
	user_id: Uuid,
	date: NaiveDate,
	status: Option<Availability>,
	note: &str,
) -> AppResult<()> {
	if status.is_none() && note.is_empty() {
		sqlx::query("DELETE FROM calendar_days WHERE user_id = $1 AND date = $2")
			.bind(user_id)
			.bind(date)
			.execute(pool)
			.await?;
		return Ok(());
	}
	sqlx::query(
		"INSERT INTO calendar_days (user_id, date, availability, note, updated_at)
		 VALUES ($1, $2, $3, $4, now())
		 ON CONFLICT (user_id, date) DO UPDATE SET
		   availability = excluded.availability,
		   note = excluded.note,
		   updated_at = excluded.updated_at",
	)
	.bind(user_id)
	.bind(date)
	.bind(status.map(|s| s.as_str()))
	.bind(note)
	.execute(pool)
	.await?;
	Ok(())
}

pub async fn set_day_status(
	pool: &PgPool,
	user_id: Uuid,
	date: NaiveDate,
	status: Option<Availability>,
) -> AppResult<()> {
	let existing_note: Option<String> =
		sqlx::query_scalar("SELECT note FROM calendar_days WHERE user_id = $1 AND date = $2")
			.bind(user_id)
			.bind(date)
			.fetch_optional(pool)
			.await?;
	upsert(pool, user_id, date, status, &existing_note.unwrap_or_default()).await
}

pub async fn set_day_note(pool: &PgPool, user_id: Uuid, date: NaiveDate, note: &str) -> AppResult<()> {
	let existing_status: Option<Option<String>> = sqlx::query_scalar(
		"SELECT availability FROM calendar_days WHERE user_id = $1 AND date = $2",
	)
	.bind(user_id)
	.bind(date)
	.fetch_optional(pool)
	.await?;
	let status = existing_status
		.flatten()
		.map(|a| Availability::try_from(a.as_str()))
		.transpose()?;
	upsert(pool, user_id, date, status, note).await
}

pub async fn clear_day(pool: &PgPool, user_id: Uuid, date: NaiveDate) -> AppResult<()> {
	sqlx::query("DELETE FROM calendar_days WHERE user_id = $1 AND date = $2")
		.bind(user_id)
		.bind(date)
		.execute(pool)
		.await?;
	Ok(())
}
