use serde_json::json;

use crate::config::Config;

/// Sends `text` to `to` via Resend, or — when no API key is configured — logs
/// `log_line` so the whole flow can still be exercised without an email
/// provider. Centralizes the "prefer Resend, fall back to console" behaviour
/// every transactional email in the app shares.
async fn send_email(config: &Config, to: &str, subject: &str, text: &str, log_line: &str) {
	let Some(api_key) = &config.resend_api_key else {
		tracing::info!("{log_line}");
		return;
	};

	let client = reqwest::Client::new();
	let res = client
		.post("https://api.resend.com/emails")
		.bearer_auth(api_key)
		.json(&json!({
			"from": config.email_from,
			"to": to,
			"subject": subject,
			"text": text,
		}))
		.send()
		.await;

	match res {
		Ok(r) if r.status().is_success() => {}
		Ok(r) => {
			let status = r.status();
			let body = r.text().await.unwrap_or_default();
			tracing::error!("[loosedays] failed to send email to {to} ({status}): {body}");
			tracing::info!("{log_line}");
		}
		Err(e) => {
			tracing::error!("[loosedays] failed to send email to {to}: {e}");
			tracing::info!("{log_line}");
		}
	}
}

/// Set RESEND_API_KEY to send real mail via Resend; otherwise the link is
/// logged to the server console so the whole flow can be tested without an
/// email provider.
pub async fn send_magic_link_email(config: &Config, to: &str, url: &str) {
	let subject = "Your Loose Days sign-in link";
	let text = format!(
		"Tap to sign in to Loose Days:\n\n{url}\n\nThis link expires in 15 minutes and can only be used once."
	);
	send_email(config, to, subject, &text, &format!("[loosedays] magic link for {to}: {url}")).await;
}

/// Invite notification for someone who isn't on Loose Days yet. Unlike the
/// magic link this carries no token — it just points them at the sign-in
/// screen, where their now-allowlisted email can request its own link.
pub async fn send_invite_email(config: &Config, to: &str, inviter_name: &str, signin_url: &str) {
	let subject = format!("{inviter_name} invited you to Loose Days");
	let text = format!(
		"{inviter_name} invited you to Loose Days — a shared calendar for finding time with friends.\n\n\
		 Sign in with this email to get started:\n\n{signin_url}\n\n\
		 We'll email you a one-time sign-in link, no password needed."
	);
	send_email(config, to, &subject, &text, &format!("[loosedays] invite for {to} from {inviter_name}: {signin_url}")).await;
}
