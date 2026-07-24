mod auth;
mod config;
mod db;
mod email;
mod error;
mod extractors;
mod handlers;
mod phone;
mod repo;
mod scheduler;
mod state;
mod types;
mod webauthn;

use axum::routing::{delete, get, patch, post};
use axum::Router;
use tower_http::trace::TraceLayer;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();
	tracing_subscriber::fmt()
		.with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse()?))
		.init();

	let config = Config::from_env();
	let pool = db::connect(&config.database_url).await?;

	// Bootstrap the founding account so there's at least one invited email
	// to sign in with on a fresh database.
	if let Some(admin_email) = &config.admin_email {
		let email = auth::normalize_email(admin_email);
		sqlx::query("INSERT INTO invited_emails (email, invited_by) VALUES ($1, NULL) ON CONFLICT (email) DO NOTHING")
			.bind(&email)
			.execute(&pool)
			.await?;
	}

	let port = config.port;
	let webauthn_instance = webauthn::build(&config)?;
	let state = AppState { pool, config, webauthn: webauthn_instance };

	// Background task: emails users a Sunday reminder to plan the coming week.
	scheduler::spawn(state.clone());

	let app = Router::new()
		.route("/api/auth/magic-link", post(handlers::auth::magic_link))
		.route("/api/auth/logout", post(handlers::auth::logout))
		.route("/auth/callback", get(handlers::auth::callback))
		.route("/api/session", get(handlers::session::get_session))
		.route("/api/auth/passkey/start", post(handlers::passkeys::auth_start))
		.route("/api/auth/passkey/finish", post(handlers::passkeys::auth_finish))
		.route(
			"/api/passkeys",
			get(handlers::passkeys::list).post(handlers::passkeys::register_start),
		)
		.route("/api/passkeys/finish", post(handlers::passkeys::register_finish))
		.route("/api/passkeys/:id", delete(handlers::passkeys::remove))
		.route("/api/onboarding/complete", post(handlers::misc::onboarding_complete))
		.route("/api/me/phone", post(handlers::misc::set_phone))
		.route("/api/invites", post(handlers::misc::invite))
		.route("/api/invite-links", post(handlers::misc::create_invite))
		.route("/api/calendar", get(handlers::calendar::get_days))
		.route(
			"/api/calendar/:date",
			patch(handlers::calendar::patch_day).delete(handlers::calendar::delete_day),
		)
		.route(
			"/api/friends",
			get(handlers::friends::list_friends).post(handlers::friends::add_friend),
		)
		.route("/api/contacts/match", post(handlers::friends::match_contacts))
		.route("/api/friends/search-email", post(handlers::friends::search_email))
		.route(
			"/api/friends/:friend_id/access-request",
			post(handlers::friends::request_access),
		)
		.route(
			"/api/friends/:friend_id/calendar",
			get(handlers::friends::friend_calendar),
		)
		.route("/api/access/incoming", get(handlers::access::incoming))
		.route("/api/access/incoming/:id/approve", post(handlers::access::approve))
		.route("/api/access/incoming/:id/deny", post(handlers::access::deny))
		.route("/api/access/mine", get(handlers::access::mine))
		.route("/api/access/outgoing", get(handlers::access::outgoing))
		.route(
			"/api/access/outgoing/:owner_id",
			delete(handlers::access::cancel_outgoing),
		)
		.route(
			"/api/access/standing",
			get(handlers::access::standing),
		)
		.route("/api/access/standing/:id", delete(handlers::access::revoke))
		.route("/api/polls", get(handlers::polls::list).post(handlers::polls::create))
		.route("/api/polls/:id", get(handlers::polls::get))
		.route("/api/polls/:id/respond", post(handlers::polls::respond))
		.route("/api/polls/:id/finalize", post(handlers::polls::finalize))
		.route("/api/polls/:id/reopen", post(handlers::polls::reopen))
		.route("/api/public/polls/:token", get(handlers::polls::public_get))
		.route(
			"/api/public/polls/:token/respond",
			post(handlers::polls::public_respond),
		)
		.layer(TraceLayer::new_for_http())
		.with_state(state);

	let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await?;
	tracing::info!("listening on 0.0.0.0:{port}");
	axum::serve(listener, app).await?;
	Ok(())
}
