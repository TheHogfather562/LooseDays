use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

pub struct NudgeTarget {
	pub id: Uuid,
	pub email: String,
	pub display_name: String,
}

/// Onboarded users who should get the Sunday "plan next week" reminder: those
/// who haven't set any availability for the coming week yet and haven't already
/// been nudged for it. Skipping anyone who's already started filling the week
/// in keeps the reminder from nagging people who don't need it, and the
/// `weekly_nudges` check makes a re-run (restart/catch-up) a no-op.
pub async fn users_to_nudge(
	pool: &PgPool,
	week_start: NaiveDate,
	week_end: NaiveDate,
) -> AppResult<Vec<NudgeTarget>> {
	let rows = sqlx::query_as::<_, (Uuid, String, String)>(
		"SELECT u.id, u.email, u.display_name
		 FROM users u
		 WHERE u.onboarded = true
		   AND NOT EXISTS (
		     SELECT 1 FROM calendar_days c
		     WHERE c.user_id = u.id
		       AND c.date BETWEEN $1 AND $2
		       AND c.availability IS NOT NULL
		   )
		   AND NOT EXISTS (
		     SELECT 1 FROM weekly_nudges n
		     WHERE n.user_id = u.id AND n.week_start = $1
		   )",
	)
	.bind(week_start)
	.bind(week_end)
	.fetch_all(pool)
	.await?;

	Ok(rows
		.into_iter()
		.map(|(id, email, display_name)| NudgeTarget { id, email, display_name })
		.collect())
}

/// Marks a user as nudged for `week_start`. Idempotent so a partial batch that
/// re-runs won't error on rows already recorded.
pub async fn record_nudge(pool: &PgPool, user_id: Uuid, week_start: NaiveDate) -> AppResult<()> {
	sqlx::query(
		"INSERT INTO weekly_nudges (user_id, week_start) VALUES ($1, $2)
		 ON CONFLICT (user_id, week_start) DO NOTHING",
	)
	.bind(user_id)
	.bind(week_start)
	.execute(pool)
	.await?;
	Ok(())
}
