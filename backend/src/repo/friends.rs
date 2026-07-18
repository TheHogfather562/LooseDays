use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::normalize_email;
use crate::error::AppResult;
use crate::phone::hash_phone;
use crate::types::{ContactDto, EmailSearchResultDto, FriendDto};

pub async fn get_friends(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<FriendDto>> {
	let rows = sqlx::query_as::<_, (Uuid, String)>(
		"SELECT u.id, u.display_name FROM friend_edges fe
		 JOIN users u ON u.id = fe.friend_id
		 WHERE fe.user_id = $1
		 ORDER BY u.display_name",
	)
	.bind(user_id)
	.fetch_all(pool)
	.await?;
	Ok(rows
		.into_iter()
		.map(|(id, display_name)| FriendDto { id, display_name })
		.collect())
}

async fn add_friend_edge(pool: &PgPool, user_id: Uuid, friend_id: Uuid) -> AppResult<()> {
	sqlx::query("INSERT INTO friend_edges (user_id, friend_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
		.bind(user_id)
		.bind(friend_id)
		.execute(pool)
		.await?;
	sqlx::query("INSERT INTO friend_edges (user_id, friend_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
		.bind(friend_id)
		.bind(user_id)
		.execute(pool)
		.await?;
	Ok(())
}

/// Matches raw phone numbers (e.g. pasted from a contacts app) against
/// existing accounts by comparing HMAC hashes — nothing here is persisted,
/// per the spec's "no phone numbers stored" rule; this is a one-off,
/// in-memory lookup for the duration of the request.
pub async fn match_contacts(
	pool: &PgPool,
	user_id: Uuid,
	entries: &[(String, String)],
	pepper: &str,
	default_region: &str,
) -> AppResult<Vec<ContactDto>> {
	let existing_friends: HashSet<Uuid> = sqlx::query_scalar(
		"SELECT friend_id FROM friend_edges WHERE user_id = $1",
	)
	.bind(user_id)
	.fetch_all(pool)
	.await?
	.into_iter()
	.collect();

	let mut out = Vec::with_capacity(entries.len());
	for (i, (name, phone)) in entries.iter().enumerate() {
		let hash = hash_phone(pepper, phone, default_region).ok();
		let matched_user: Option<(Uuid,)> = match &hash {
			Some(h) => sqlx::query_as("SELECT id FROM users WHERE phone_hash = $1")
				.bind(h)
				.fetch_optional(pool)
				.await?,
			None => None,
		};
		let matched = matched_user.as_ref().is_some_and(|(id,)| *id != user_id);
		let matched_id = matched.then(|| matched_user.unwrap().0);
		out.push(ContactDto {
			id: format!("c{i}_{}", hash.unwrap_or_default()),
			name: if name.is_empty() { phone.clone() } else { name.clone() },
			phone: phone.clone(),
			matched,
			user_id: matched_id,
			already_friend: matched_id.is_some_and(|id| existing_friends.contains(&id)),
		});
	}
	Ok(out)
}

/// Looks up an account by email (case/whitespace-insensitive, matching how
/// emails are normalized at sign-up) so a friend can be found without
/// needing their phone number in contacts first.
pub async fn search_by_email(
	pool: &PgPool,
	user_id: Uuid,
	email: &str,
) -> AppResult<EmailSearchResultDto> {
	let normalized = normalize_email(email);
	let found: Option<(Uuid, String)> =
		sqlx::query_as("SELECT id, display_name FROM users WHERE email = $1")
			.bind(&normalized)
			.fetch_optional(pool)
			.await?;

	let Some((id, display_name)) = found.filter(|(id, _)| *id != user_id) else {
		return Ok(EmailSearchResultDto {
			found: false,
			user_id: None,
			display_name: None,
			already_friend: false,
		});
	};

	let already_friend: Option<(Uuid,)> = sqlx::query_as(
		"SELECT friend_id FROM friend_edges WHERE user_id = $1 AND friend_id = $2",
	)
	.bind(user_id)
	.bind(id)
	.fetch_optional(pool)
	.await?;

	Ok(EmailSearchResultDto {
		found: true,
		user_id: Some(id),
		display_name: Some(display_name),
		already_friend: already_friend.is_some(),
	})
}

pub async fn add_friend(pool: &PgPool, user_id: Uuid, friend_user_id: Uuid) -> AppResult<()> {
	if user_id == friend_user_id {
		return Ok(());
	}
	let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
		.bind(friend_user_id)
		.fetch_optional(pool)
		.await?;
	if exists.is_none() {
		return Ok(());
	}
	add_friend_edge(pool, user_id, friend_user_id).await
}
