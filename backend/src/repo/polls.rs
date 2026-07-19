use std::collections::HashMap;

use chrono::NaiveDate;
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::phone::hash_phone;
use crate::types::{Availability, CreatePollInput, InviteeStatus, PollDto, PollInviteeDto};

#[derive(sqlx::FromRow)]
struct PollRow {
	id: Uuid,
	creator_id: Uuid,
	title: String,
	note: String,
	range_start: NaiveDate,
	range_end: NaiveDate,
	finalized_start: Option<NaiveDate>,
	finalized_end: Option<NaiveDate>,
	finalized_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Every read path selects the same poll columns; keep the list in one place so
// PollRow's shape and the queries can't drift.
const POLL_COLS: &str =
	"id, creator_id, title, note, range_start, range_end, finalized_start, finalized_end, finalized_at";

#[derive(sqlx::FromRow)]
struct InviteeRow {
	id: Uuid,
	user_id: Option<Uuid>,
	name: String,
	access_token: String,
	status: String,
}

fn new_token() -> String {
	let mut bytes = [0u8; 9];
	rand::thread_rng().fill_bytes(&mut bytes);
	base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

/// `include_all_tokens` controls whether every invitee's `access_token` is
/// serialized, or only the viewer's own. The tokens are bearer credentials
/// for responding to a poll, so they're only ever handed out wholesale to the
/// creator right after creation (to build the invite links they send out) —
/// never on a read path, where leaking them would let any one invitee harvest
/// every other invitee's token and overwrite/read their responses.
async fn build_poll(
	pool: &PgPool,
	poll: PollRow,
	viewer_invitee_id: Option<Uuid>,
	include_all_tokens: bool,
) -> AppResult<PollDto> {
	let invitees = sqlx::query_as::<_, InviteeRow>(
		"SELECT id, user_id, name, access_token, status FROM poll_invitees WHERE poll_id = $1 ORDER BY created_at",
	)
	.bind(poll.id)
	.fetch_all(pool)
	.await?;

	let response_rows = sqlx::query_as::<_, (Uuid, NaiveDate, String)>(
		"SELECT invitee_id, date, availability FROM poll_responses WHERE poll_id = $1",
	)
	.bind(poll.id)
	.fetch_all(pool)
	.await?;

	let mut responses: HashMap<Uuid, HashMap<NaiveDate, Availability>> = HashMap::new();
	for (invitee_id, date, availability) in response_rows {
		responses
			.entry(invitee_id)
			.or_default()
			.insert(date, Availability::try_from(availability.as_str())?);
	}

	let dto_invitees = invitees
		.into_iter()
		.map(|inv| {
			Ok(PollInviteeDto {
				id: inv.id,
				user_id: inv.user_id,
				phone: None,
				name: inv.name,
				status: InviteeStatus::try_from(inv.status)?,
				access_token: if include_all_tokens || Some(inv.id) == viewer_invitee_id {
					Some(inv.access_token)
				} else {
					None
				},
				is_me: Some(inv.id) == viewer_invitee_id,
			})
		})
		.collect::<AppResult<Vec<_>>>()?;

	Ok(PollDto {
		id: poll.id,
		title: poll.title,
		note: poll.note,
		creator_id: poll.creator_id,
		range_start: poll.range_start,
		range_end: poll.range_end,
		finalized_start: poll.finalized_start,
		finalized_end: poll.finalized_end,
		finalized_at: poll.finalized_at,
		invitees: dto_invitees,
		responses,
	})
}

pub async fn create_poll(
	pool: &PgPool,
	creator_id: Uuid,
	input: CreatePollInput,
	pepper: &str,
	default_region: &str,
) -> AppResult<PollDto> {
	let title = if input.title.is_empty() { "Untitled poll".to_string() } else { input.title };

	let poll_id: Uuid = sqlx::query_scalar(
		"INSERT INTO polls (creator_id, title, note, range_start, range_end)
		 VALUES ($1, $2, $3, $4, $5) RETURNING id",
	)
	.bind(creator_id)
	.bind(&title)
	.bind(&input.note)
	.bind(input.start)
	.bind(input.end)
	.fetch_one(pool)
	.await?;

	let creator_name: String = sqlx::query_scalar("SELECT display_name FROM users WHERE id = $1")
		.bind(creator_id)
		.fetch_one(pool)
		.await?;

	let my_invitee_id: Uuid = sqlx::query_scalar(
		"INSERT INTO poll_invitees (poll_id, user_id, name, access_token) VALUES ($1, $2, $3, $4) RETURNING id",
	)
	.bind(poll_id)
	.bind(creator_id)
	.bind(&creator_name)
	.bind(new_token())
	.fetch_one(pool)
	.await?;

	if !input.friend_ids.is_empty() {
		let friends = sqlx::query_as::<_, (Uuid, String)>(
			"SELECT id, display_name FROM users WHERE id = ANY($1)",
		)
		.bind(&input.friend_ids)
		.fetch_all(pool)
		.await?;
		for (id, display_name) in friends {
			sqlx::query(
				"INSERT INTO poll_invitees (poll_id, user_id, name, access_token) VALUES ($1, $2, $3, $4)",
			)
			.bind(poll_id)
			.bind(id)
			.bind(display_name)
			.bind(new_token())
			.execute(pool)
			.await?;
		}
	}

	// Raw phone numbers are never persisted — only the hash, per spec. The
	// name must be client-supplied (no more falling back to the phone
	// digits as a display name) since the server can't reconstruct it later.
	for invitee in &input.phone_invitees {
		let hash = hash_phone(pepper, &invitee.phone, default_region).ok();
		let name = if invitee.name.trim().is_empty() { "Guest".to_string() } else { invitee.name.clone() };
		sqlx::query(
			"INSERT INTO poll_invitees (poll_id, phone_hash, name, access_token) VALUES ($1, $2, $3, $4)",
		)
		.bind(poll_id)
		.bind(&hash)
		.bind(&name)
		.bind(new_token())
		.execute(pool)
		.await?;
	}

	let poll_row = sqlx::query_as::<_, PollRow>(
		&format!("SELECT {POLL_COLS} FROM polls WHERE id = $1"),
	)
	.bind(poll_id)
	.fetch_one(pool)
	.await?;

	// Creation is the one place the creator legitimately needs every
	// invitee's token — to build the SMS/WhatsApp invite links they send out.
	let mut dto = build_poll(pool, poll_row, Some(my_invitee_id), true).await?;

	// Echo back the just-submitted raw phones on the invitees that came from
	// phoneInvitees, in this response only, so the "send invites" screen can
	// build SMS/WhatsApp deep links immediately without a round trip to
	// fetch something the server intentionally never stored.
	let mut phone_by_name: HashMap<String, String> = HashMap::new();
	for invitee in &input.phone_invitees {
		let name = if invitee.name.trim().is_empty() { "Guest".to_string() } else { invitee.name.clone() };
		phone_by_name.insert(name, invitee.phone.clone());
	}
	for inv in &mut dto.invitees {
		if inv.user_id.is_none() {
			inv.phone = phone_by_name.get(&inv.name).cloned();
		}
	}

	Ok(dto)
}

pub async fn list_polls_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<PollDto>> {
	let rows = sqlx::query_as::<_, PollRow>(
		&format!(
			"SELECT {POLL_COLS}
			 FROM polls
			 WHERE creator_id = $1 OR id IN (SELECT poll_id FROM poll_invitees WHERE user_id = $1)
			 ORDER BY created_at DESC"
		),
	)
	.bind(user_id)
	.fetch_all(pool)
	.await?;

	let mut out = Vec::with_capacity(rows.len());
	for row in rows {
		let mine: Option<Uuid> = sqlx::query_scalar(
			"SELECT id FROM poll_invitees WHERE poll_id = $1 AND user_id = $2",
		)
		.bind(row.id)
		.bind(user_id)
		.fetch_optional(pool)
		.await?;
		out.push(build_poll(pool, row, mine, false).await?);
	}
	Ok(out)
}

pub async fn get_poll_for_user(pool: &PgPool, poll_id: Uuid, user_id: Uuid) -> AppResult<Option<PollDto>> {
	let Some(row) = sqlx::query_as::<_, PollRow>(
		&format!("SELECT {POLL_COLS} FROM polls WHERE id = $1"),
	)
	.bind(poll_id)
	.fetch_optional(pool)
	.await?
	else {
		return Ok(None);
	};

	let mine: Option<Uuid> = sqlx::query_scalar(
		"SELECT id FROM poll_invitees WHERE poll_id = $1 AND user_id = $2",
	)
	.bind(poll_id)
	.bind(user_id)
	.fetch_optional(pool)
	.await?;

	if row.creator_id != user_id && mine.is_none() {
		return Ok(None);
	}
	Ok(Some(build_poll(pool, row, mine, false).await?))
}

/// Record the range the group settled on. Creator-only; the `creator_id`
/// predicate means a non-creator (or unknown poll) touches no rows and gets
/// `false` back, which the handler turns into a 403/404. A single chosen day
/// is passed as start == end.
pub async fn finalize_poll(
	pool: &PgPool,
	poll_id: Uuid,
	creator_id: Uuid,
	start: NaiveDate,
	end: NaiveDate,
) -> AppResult<bool> {
	let (start, end) = if end < start { (end, start) } else { (start, end) };
	let affected = sqlx::query(
		"UPDATE polls
		 SET finalized_start = $1, finalized_end = $2, finalized_at = now()
		 WHERE id = $3 AND creator_id = $4",
	)
	.bind(start)
	.bind(end)
	.bind(poll_id)
	.bind(creator_id)
	.execute(pool)
	.await?;
	Ok(affected.rows_affected() > 0)
}

/// Undo a finalize, back to an open poll. Creator-only, same as finalize.
pub async fn reopen_poll(pool: &PgPool, poll_id: Uuid, creator_id: Uuid) -> AppResult<bool> {
	let affected = sqlx::query(
		"UPDATE polls
		 SET finalized_start = NULL, finalized_end = NULL, finalized_at = NULL
		 WHERE id = $1 AND creator_id = $2",
	)
	.bind(poll_id)
	.bind(creator_id)
	.execute(pool)
	.await?;
	Ok(affected.rows_affected() > 0)
}

async fn save_responses(
	pool: &PgPool,
	poll_id: Uuid,
	invitee_id: Uuid,
	responses: &HashMap<NaiveDate, Availability>,
) -> AppResult<()> {
	let mut tx = pool.begin().await?;
	sqlx::query("DELETE FROM poll_responses WHERE poll_id = $1 AND invitee_id = $2")
		.bind(poll_id)
		.bind(invitee_id)
		.execute(&mut *tx)
		.await?;
	for (date, availability) in responses {
		sqlx::query(
			"INSERT INTO poll_responses (poll_id, invitee_id, date, availability) VALUES ($1, $2, $3, $4)",
		)
		.bind(poll_id)
		.bind(invitee_id)
		.bind(date)
		.bind(availability.as_str())
		.execute(&mut *tx)
		.await?;
	}
	sqlx::query("UPDATE poll_invitees SET status = 'responded' WHERE id = $1")
		.bind(invitee_id)
		.execute(&mut *tx)
		.await?;
	tx.commit().await?;
	Ok(())
}

pub async fn submit_response(
	pool: &PgPool,
	poll_id: Uuid,
	user_id: Uuid,
	responses: &HashMap<NaiveDate, Availability>,
) -> AppResult<bool> {
	let invitee: Option<Uuid> = sqlx::query_scalar(
		"SELECT id FROM poll_invitees WHERE poll_id = $1 AND user_id = $2",
	)
	.bind(poll_id)
	.bind(user_id)
	.fetch_optional(pool)
	.await?;
	let Some(invitee_id) = invitee else {
		return Ok(false);
	};
	save_responses(pool, poll_id, invitee_id, responses).await?;
	Ok(true)
}

pub async fn get_poll_by_token(pool: &PgPool, token: &str) -> AppResult<Option<PollDto>> {
	let invitee: Option<(Uuid, Uuid)> = sqlx::query_as(
		"SELECT id, poll_id FROM poll_invitees WHERE access_token = $1",
	)
	.bind(token)
	.fetch_optional(pool)
	.await?;
	let Some((invitee_id, poll_id)) = invitee else {
		return Ok(None);
	};
	let Some(row) = sqlx::query_as::<_, PollRow>(
		&format!("SELECT {POLL_COLS} FROM polls WHERE id = $1"),
	)
	.bind(poll_id)
	.fetch_optional(pool)
	.await?
	else {
		return Ok(None);
	};
	Ok(Some(build_poll(pool, row, Some(invitee_id), false).await?))
}

pub async fn submit_response_by_token(
	pool: &PgPool,
	token: &str,
	responses: &HashMap<NaiveDate, Availability>,
) -> AppResult<bool> {
	let invitee: Option<(Uuid, Uuid)> = sqlx::query_as(
		"SELECT id, poll_id FROM poll_invitees WHERE access_token = $1",
	)
	.bind(token)
	.fetch_optional(pool)
	.await?;
	let Some((invitee_id, poll_id)) = invitee else {
		return Ok(false);
	};
	save_responses(pool, poll_id, invitee_id, responses).await?;
	Ok(true)
}
