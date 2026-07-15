use std::env;

#[derive(Clone)]
pub struct Config {
	pub database_url: String,
	pub server_pepper: String,
	pub admin_email: Option<String>,
	pub resend_api_key: Option<String>,
	pub email_from: String,
	pub port: u16,
	/// Absolute origin (scheme + host) used to build the magic-link URL that
	/// gets emailed out. Not derivable from the request in a way that's safe
	/// behind a tunnel/proxy, so it's configured explicitly.
	pub public_origin: String,
	/// Region used to interpret phone numbers with no country code, e.g. "US".
	pub default_phone_region: String,
	pub cookie_secure: bool,
}

impl Config {
	pub fn from_env() -> Self {
		Self {
			database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
			server_pepper: env::var("SERVER_PEPPER").expect("SERVER_PEPPER must be set"),
			admin_email: env::var("ADMIN_EMAIL").ok().filter(|s| !s.is_empty()),
			resend_api_key: env::var("RESEND_API_KEY").ok().filter(|s| !s.is_empty()),
			email_from: env::var("EMAIL_FROM")
				.unwrap_or_else(|_| "Loose Days <onboarding@resend.dev>".to_string()),
			port: env::var("PORT")
				.ok()
				.and_then(|p| p.parse().ok())
				.unwrap_or(8080),
			public_origin: env::var("PUBLIC_ORIGIN")
				.unwrap_or_else(|_| "http://localhost:8080".to_string()),
			default_phone_region: env::var("DEFAULT_PHONE_REGION").unwrap_or_else(|_| "US".to_string()),
			cookie_secure: env::var("COOKIE_SECURE")
				.map(|v| v == "true")
				.unwrap_or_else(|_| !env::var("PUBLIC_ORIGIN").unwrap_or_default().starts_with("http://")),
		}
	}
}
