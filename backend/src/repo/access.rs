use std::collections::HashMap;

use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::types::{
	AccessScope, Availability, DetailLevel, FriendAccessGrantDto, IncomingRequestDto,
	StandingAccessGrantDto,
};

pub async fn request_access(
	pool: &PgPool,
	requester_id: Uuid,
	owner_id: Uuid,
	scope: AccessScope,
	range_start: Option<NaiveDate>,
	range_end: Option<NaiveDate>,
) -> AppResult<()> {
	sqlx::query(
		"INSERT INTO calendar_access_requests (requester_id, owner_id, scope, range_start, range_end, status)
		 VALUES ($1, $2, $3, $4, $5, 'pending')",
	)
	.bind(requester_id)
	.bind(owner_id)
	.bind(scope.as_str())
	.bind(range_start)
	.bind(range_end)
	.execute(pool)
	.await?;
	Ok(())
}

/// Withdraw the requester's own still-pending access request(s) against a
/// given owner. Only pending rows are touched, so this can't retract an
/// already-approved grant (use revoke for that, from the owner's side).
pub async fn cancel_outgoing(pool: &PgPool, requester_id: Uuid, owner_id: Uuid) -> AppResult<()> {
	sqlx::query(
		"DELETE FROM calendar_access_requests
		 WHERE requester_id = $1 AND owner_id = $2 AND status = 'pending'",
	)
	.bind(requester_id)
	.bind(owner_id)
	.execute(pool)
	.await?;
	Ok(())
}

pub async fn get_incoming_requests(pool: &PgPool, owner_id: Uuid) -> AppResult<Vec<IncomingRequestDto>> {
	let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String, Option<NaiveDate>, Option<NaiveDate>)>(
		"SELECT r.id, r.requester_id, u.display_name, r.scope, r.range_start, r.range_end
		 FROM calendar_access_requests r
		 JOIN users u ON u.id = r.requester_id
		 WHERE r.owner_id = $1 AND r.status = 'pending'
		 ORDER BY r.created_at",
	)
	.bind(owner_id)
	.fetch_all(pool)
	.await?;

	rows.into_iter()
		.map(|(id, requester_id, requester_name, scope, range_start, range_end)| {
			Ok(IncomingRequestDto {
				id,
				requester_id,
				requester_name,
				scope: AccessScope::try_from(scope)?,
				range_start,
				range_end,
			})
		})
		.collect()
}

pub async fn approve_request(
	pool: &PgPool,
	request_id: Uuid,
	owner_id: Uuid,
	detail_level: DetailLevel,
) -> AppResult<()> {
	sqlx::query(
		"UPDATE calendar_access_requests
		 SET status = 'approved', detail_level = $1, responded_at = now()
		 WHERE id = $2 AND owner_id = $3 AND status = 'pending'",
	)
	.bind(detail_level.as_str())
	.bind(request_id)
	.bind(owner_id)
	.execute(pool)
	.await?;
	Ok(())
}

pub async fn deny_request(pool: &PgPool, request_id: Uuid, owner_id: Uuid) -> AppResult<()> {
	sqlx::query(
		"UPDATE calendar_access_requests
		 SET status = 'denied', responded_at = now()
		 WHERE id = $1 AND owner_id = $2 AND status = 'pending'",
	)
	.bind(request_id)
	.bind(owner_id)
	.execute(pool)
	.await?;
	Ok(())
}

pub async fn revoke_access(pool: &PgPool, request_id: Uuid, owner_id: Uuid) -> AppResult<()> {
	sqlx::query(
		"UPDATE calendar_access_requests
		 SET status = 'revoked', responded_at = now()
		 WHERE id = $1 AND owner_id = $2 AND status = 'approved'",
	)
	.bind(request_id)
	.bind(owner_id)
	.execute(pool)
	.await?;
	Ok(())
}

/// Access I (the requester) have been granted into friends' calendars, keyed
/// by friend id. When more than one grant exists for the same friend, the
/// most recently-approved one wins.
pub async fn get_my_access(pool: &PgPool, requester_id: Uuid) -> AppResult<HashMap<Uuid, FriendAccessGrantDto>> {
	let rows = sqlx::query_as::<_, (Uuid, String, String, Option<NaiveDate>, Option<NaiveDate>)>(
		"SELECT owner_id, scope, detail_level, range_start, range_end
		 FROM calendar_access_requests
		 WHERE requester_id = $1 AND status = 'approved'
		 ORDER BY responded_at ASC",
	)
	.bind(requester_id)
	.fetch_all(pool)
	.await?;

	let mut out = HashMap::new();
	for (owner_id, scope, detail_level, range_start, range_end) in rows {
		out.insert(
			owner_id,
			FriendAccessGrantDto {
				level: if detail_level == "full" { "full".into() } else { "overlap".into() },
				scope: AccessScope::try_from(scope)?,
				range_start,
				range_end,
			},
		);
	}
	Ok(out)
}

pub async fn get_outgoing_pending(pool: &PgPool, requester_id: Uuid) -> AppResult<HashMap<Uuid, bool>> {
	let rows: Vec<Uuid> = sqlx::query_scalar(
		"SELECT DISTINCT owner_id FROM calendar_access_requests WHERE requester_id = $1 AND status = 'pending'",
	)
	.bind(requester_id)
	.fetch_all(pool)
	.await?;
	Ok(rows.into_iter().map(|id| (id, true)).collect())
}

/// Everyone who currently has approved access into my calendar.
pub async fn get_standing_access(pool: &PgPool, owner_id: Uuid) -> AppResult<Vec<StandingAccessGrantDto>> {
	let rows = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
		"SELECT r.id, r.requester_id, u.display_name, r.detail_level
		 FROM calendar_access_requests r
		 JOIN users u ON u.id = r.requester_id
		 WHERE r.owner_id = $1 AND r.status = 'approved'
		 ORDER BY r.responded_at DESC",
	)
	.bind(owner_id)
	.fetch_all(pool)
	.await?;

	rows.into_iter()
		.map(|(id, friend_id, friend_name, detail_level)| {
			Ok(StandingAccessGrantDto {
				id,
				friend_id,
				friend_name,
				detail_level: DetailLevel::try_from(detail_level)?,
			})
		})
		.collect()
}

/// What `viewer_id` is allowed to see of `owner_id`'s calendar, already
/// filtered to the granted date range and detail level — full status, or (in
/// overlap mode) only the dates the owner marked free, with no other detail
/// leaked. Returns {} if there's no active grant.
pub async fn get_visible_friend_calendar(
	pool: &PgPool,
	viewer_id: Uuid,
	owner_id: Uuid,
) -> AppResult<HashMap<NaiveDate, Availability>> {
	let grant = sqlx::query_as::<_, (String, String, Option<NaiveDate>, Option<NaiveDate>)>(
		"SELECT scope, detail_level, range_start, range_end
		 FROM calendar_access_requests
		 WHERE requester_id = $1 AND owner_id = $2 AND status = 'approved'
		 ORDER BY responded_at DESC LIMIT 1",
	)
	.bind(viewer_id)
	.bind(owner_id)
	.fetch_optional(pool)
	.await?;

	let Some((scope, detail_level, range_start, range_end)) = grant else {
		return Ok(HashMap::new());
	};

	let rows = sqlx::query_as::<_, (NaiveDate, String)>(
		"SELECT date, availability FROM calendar_days WHERE user_id = $1 AND availability IS NOT NULL",
	)
	.bind(owner_id)
	.fetch_all(pool)
	.await?;

	let in_range = |date: NaiveDate| {
		scope != "range"
			|| (range_start.is_none_or(|s| date >= s) && range_end.is_none_or(|e| date <= e))
	};

	let mut out = HashMap::new();
	for (date, availability) in rows {
		if !in_range(date) {
			continue;
		}
		let availability = Availability::try_from(availability.as_str())?;
		if detail_level == "full" {
			out.insert(date, availability);
		} else if availability == Availability::Free {
			out.insert(date, Availability::Free);
		}
	}
	Ok(out)
}
