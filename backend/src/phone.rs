use std::str::FromStr;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{AppError, AppResult};

/// Normalizes to E.164 using the given default region for numbers with no
/// country code, per spec ("Normalize to E.164 (libphonenumber) before
/// hashing."). Falls back to a digits-only string if the input can't be
/// parsed as a phone number at all, so a malformed entry still hashes to
/// *something* stable rather than hard-failing the whole request.
pub fn normalize_e164(raw: &str, default_region: &str) -> String {
	let country = phonenumber::country::Id::from_str(default_region).ok();
	match phonenumber::parse(country, raw) {
		Ok(number) => number.format().mode(phonenumber::Mode::E164).to_string(),
		Err(_) => raw.chars().filter(|c| c.is_ascii_digit()).collect(),
	}
}

/// HMAC-SHA256(SERVER_PEPPER, E164(number)) — a single server-side pepper so
/// friend A's hash of friend B's number matches B's own hash of themselves.
pub fn hash_phone(pepper: &str, raw: &str, default_region: &str) -> AppResult<String> {
	let normalized = normalize_e164(raw, default_region);
	if normalized.is_empty() {
		return Err(AppError::BadRequest("invalid phone number".into()));
	}
	let mut mac = Hmac::<Sha256>::new_from_slice(pepper.as_bytes())
		.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
	mac.update(normalized.as_bytes());
	Ok(hex::encode(mac.finalize().into_bytes()))
}
