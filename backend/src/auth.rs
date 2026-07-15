use chrono::{Duration, Utc};
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::phone::hash_phone;
use crate::types::SessionUserDto;

pub const SESSION_COOKIE: &str = "ld_session";
const SESSION_TTL_DAYS: i64 = 30;
const MAGIC_LINK_TTL_MINUTES: i64 = 15;

pub fn normalize_email(email: &str) -> String {
	email.trim().to_lowercase()
}

fn new_token() -> String {
	let mut bytes = [0u8; 32];
	rand::thread_rng().fill_bytes(&mut bytes);
	hex::encode(bytes)
}

#[derive(sqlx::FromRow)]
struct UserRow {
	id: Uuid,
	email: String,
	phone_hash: Option<String>,
	display_name: String,
	onboarded: bool,
}

impl From<UserRow> for SessionUserDto {
	fn from(row: UserRow) -> Self {
		SessionUserDto {
			id: row.id,
			email: row.email,
			display_name: row.display_name,
			phone_set: row.phone_hash.is_some(),
			onboarded: row.onboarded,
		}
	}
}

/// Sign-up is invite-only: allow a magic link only for existing users or
/// emails someone already on Loose Days has invited.
pub async fn is_email_allowed(pool: &PgPool, email: &str) -> AppResult<bool> {
	let e = normalize_email(email);
	let exists_user = sqlx::query_scalar::<_, i32>("SELECT 1 FROM users WHERE email = $1")
		.bind(&e)
		.fetch_optional(pool)
		.await?;
	if exists_user.is_some() {
		return Ok(true);
	}
	let invited = sqlx::query_scalar::<_, i32>("SELECT 1 FROM invited_emails WHERE email = $1")
		.bind(&e)
		.fetch_optional(pool)
		.await?;
	Ok(invited.is_some())
}

pub async fn invite_email(pool: &PgPool, email: &str, invited_by: Uuid) -> AppResult<()> {
	let e = normalize_email(email);
	sqlx::query(
		"INSERT INTO invited_emails (email, invited_by) VALUES ($1, $2) ON CONFLICT (email) DO NOTHING",
	)
	.bind(&e)
	.bind(invited_by)
	.execute(pool)
	.await?;
	Ok(())
}

pub async fn create_magic_link(pool: &PgPool, email: &str) -> AppResult<String> {
	let token = new_token();
	let expires_at = Utc::now() + Duration::minutes(MAGIC_LINK_TTL_MINUTES);
	sqlx::query("INSERT INTO login_tokens (token, email, expires_at) VALUES ($1, $2, $3)")
		.bind(&token)
		.bind(normalize_email(email))
		.bind(expires_at)
		.execute(pool)
		.await?;
	Ok(token)
}

/// Verifies + burns a magic-link token, returning the email it was issued to.
pub async fn consume_magic_link(pool: &PgPool, token: &str) -> AppResult<Option<String>> {
	let row = sqlx::query_as::<_, (String, chrono::DateTime<Utc>, Option<chrono::DateTime<Utc>>)>(
		"SELECT email, expires_at, used_at FROM login_tokens WHERE token = $1",
	)
	.bind(token)
	.fetch_optional(pool)
	.await?;

	let Some((email, expires_at, used_at)) = row else {
		return Ok(None);
	};
	if used_at.is_some() || expires_at < Utc::now() {
		return Ok(None);
	}
	sqlx::query("UPDATE login_tokens SET used_at = now() WHERE token = $1")
		.bind(token)
		.execute(pool)
		.await?;
	Ok(Some(email))
}

pub async fn find_or_create_user_by_email(pool: &PgPool, email: &str) -> AppResult<SessionUserDto> {
	let e = normalize_email(email);
	if let Some(row) = sqlx::query_as::<_, UserRow>(
		"SELECT id, email, phone_hash, display_name, onboarded FROM users WHERE email = $1",
	)
	.bind(&e)
	.fetch_optional(pool)
	.await?
	{
		return Ok(row.into());
	}

	let display_name = e.split('@').next().unwrap_or(&e).to_string();
	let row = sqlx::query_as::<_, UserRow>(
		"INSERT INTO users (email, display_name) VALUES ($1, $2)
		 RETURNING id, email, phone_hash, display_name, onboarded",
	)
	.bind(&e)
	.bind(&display_name)
	.fetch_one(pool)
	.await?;
	Ok(row.into())
}

pub async fn create_session(pool: &PgPool, user_id: Uuid) -> AppResult<(Uuid, chrono::DateTime<Utc>)> {
	let expires_at = Utc::now() + Duration::days(SESSION_TTL_DAYS);
	let id: Uuid = sqlx::query_scalar(
		"INSERT INTO sessions (user_id, expires_at) VALUES ($1, $2) RETURNING id",
	)
	.bind(user_id)
	.bind(expires_at)
	.fetch_one(pool)
	.await?;
	Ok((id, expires_at))
}

pub async fn destroy_session(pool: &PgPool, session_id: Uuid) -> AppResult<()> {
	sqlx::query("DELETE FROM sessions WHERE id = $1")
		.bind(session_id)
		.execute(pool)
		.await?;
	Ok(())
}

pub async fn get_user_by_session(pool: &PgPool, session_id: Uuid) -> AppResult<Option<SessionUserDto>> {
	let row = sqlx::query_as::<_, UserRow>(
		"SELECT u.id, u.email, u.phone_hash, u.display_name, u.onboarded
		 FROM sessions s JOIN users u ON u.id = s.user_id
		 WHERE s.id = $1 AND s.expires_at > now()",
	)
	.bind(session_id)
	.fetch_optional(pool)
	.await?;
	Ok(row.map(Into::into))
}

/// Normalizes + hashes the phone before ever touching the DB — the raw
/// number never gets persisted, per the spec's trust model.
pub async fn set_user_phone(
	pool: &PgPool,
	user_id: Uuid,
	raw_phone: &str,
	pepper: &str,
	default_region: &str,
) -> AppResult<()> {
	let hash = hash_phone(pepper, raw_phone, default_region)?;
	let result = sqlx::query("UPDATE users SET phone_hash = $1 WHERE id = $2")
		.bind(&hash)
		.bind(user_id)
		.execute(pool)
		.await;
	match result {
		Ok(_) => Ok(()),
		Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(AppError::Conflict(
			"that phone number is already claimed by another account".into(),
		)),
		Err(e) => Err(e.into()),
	}
}

pub async fn complete_onboarding(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
	sqlx::query("UPDATE users SET onboarded = true WHERE id = $1")
		.bind(user_id)
		.execute(pool)
		.await?;
	Ok(())
}
