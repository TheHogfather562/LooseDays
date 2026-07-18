use chrono::{Duration, Utc};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;
use webauthn_rs::prelude::Passkey;

use crate::error::AppResult;
use crate::types::PasskeyDto;

const CHALLENGE_TTL_MINUTES: i64 = 5;

/// Stores in-flight ceremony state (a `PasskeyRegistration` or
/// `DiscoverableAuthentication`) keyed by a fresh opaque id, so the browser
/// only ever round-trips that id between the start/finish calls.
pub async fn store_challenge<T: Serialize>(
	pool: &PgPool,
	kind: &str,
	user_id: Option<Uuid>,
	state: &T,
) -> AppResult<Uuid> {
	let state_data = serde_json::to_value(state).map_err(anyhow::Error::from)?;
	let expires_at = Utc::now() + Duration::minutes(CHALLENGE_TTL_MINUTES);
	let id: Uuid = sqlx::query_scalar(
		"INSERT INTO webauthn_challenges (kind, user_id, state_data, expires_at) VALUES ($1, $2, $3, $4) RETURNING id",
	)
	.bind(kind)
	.bind(user_id)
	.bind(state_data)
	.bind(expires_at)
	.fetch_one(pool)
	.await?;
	Ok(id)
}

/// Consumes (deletes) the ceremony state for `id`, returning `None` if it
/// never existed, was already used, or expired.
pub async fn take_challenge<T: DeserializeOwned>(
	pool: &PgPool,
	id: Uuid,
	kind: &str,
) -> AppResult<Option<(Option<Uuid>, T)>> {
	let row = sqlx::query_as::<_, (Option<Uuid>, Value, chrono::DateTime<Utc>)>(
		"DELETE FROM webauthn_challenges WHERE id = $1 AND kind = $2 RETURNING user_id, state_data, expires_at",
	)
	.bind(id)
	.bind(kind)
	.fetch_optional(pool)
	.await?;
	let Some((user_id, state_data, expires_at)) = row else {
		return Ok(None);
	};
	if expires_at < Utc::now() {
		return Ok(None);
	}
	let state = serde_json::from_value(state_data).map_err(anyhow::Error::from)?;
	Ok(Some((user_id, state)))
}

pub async fn insert_credential(
	pool: &PgPool,
	user_id: Uuid,
	credential_id: &[u8],
	passkey: &Passkey,
	label: &str,
) -> AppResult<()> {
	let passkey_data = serde_json::to_value(passkey).map_err(anyhow::Error::from)?;
	sqlx::query(
		"INSERT INTO passkey_credentials (user_id, credential_id, passkey_data, label)
		 VALUES ($1, $2, $3, $4)",
	)
	.bind(user_id)
	.bind(credential_id)
	.bind(passkey_data)
	.bind(label)
	.execute(pool)
	.await?;
	Ok(())
}

pub async fn list_credentials(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<PasskeyDto>> {
	let rows = sqlx::query_as::<_, PasskeyDto>(
		"SELECT id, label, created_at, last_used_at FROM passkey_credentials
		 WHERE user_id = $1 ORDER BY created_at",
	)
	.bind(user_id)
	.fetch_all(pool)
	.await?;
	Ok(rows)
}

/// All of a user's stored passkeys, used both to exclude already-registered
/// authenticators during a new registration and to verify a sign-in attempt.
pub async fn load_passkeys_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Passkey>> {
	let rows = sqlx::query_scalar::<_, Value>("SELECT passkey_data FROM passkey_credentials WHERE user_id = $1")
		.bind(user_id)
		.fetch_all(pool)
		.await?;
	let mut passkeys = Vec::with_capacity(rows.len());
	for row in rows {
		passkeys.push(serde_json::from_value(row).map_err(anyhow::Error::from)?);
	}
	Ok(passkeys)
}

/// Persists the updated counter/backup-state after a successful sign-in and
/// bumps `last_used_at`.
pub async fn update_after_auth(pool: &PgPool, user_id: Uuid, credential_id: &[u8], passkey: &Passkey) -> AppResult<()> {
	let passkey_data = serde_json::to_value(passkey).map_err(anyhow::Error::from)?;
	sqlx::query(
		"UPDATE passkey_credentials SET passkey_data = $1, last_used_at = now()
		 WHERE user_id = $2 AND credential_id = $3",
	)
	.bind(passkey_data)
	.bind(user_id)
	.bind(credential_id)
	.execute(pool)
	.await?;
	Ok(())
}

/// Deletes a credential, scoped to `user_id` so one account can't remove
/// another's passkey. Returns whether a row was actually deleted.
pub async fn delete_credential(pool: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
	let result = sqlx::query("DELETE FROM passkey_credentials WHERE id = $1 AND user_id = $2")
		.bind(id)
		.bind(user_id)
		.execute(pool)
		.await?;
	Ok(result.rows_affected() > 0)
}
