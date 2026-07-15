use serde_json::json;

use crate::config::Config;

/// Pluggable email sender. Set RESEND_API_KEY to send real mail via Resend;
/// otherwise the link is logged to the server console so the whole flow can
/// be tested without an email provider.
pub async fn send_magic_link_email(config: &Config, to: &str, url: &str) {
	let subject = "Your Loose Days sign-in link";
	let text = format!(
		"Tap to sign in to Loose Days:\n\n{url}\n\nThis link expires in 15 minutes and can only be used once."
	);

	let Some(api_key) = &config.resend_api_key else {
		tracing::info!("[loosedays] magic link for {to}: {url}");
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
			tracing::error!("[loosedays] failed to send magic link email ({status}): {body}");
			tracing::info!("[loosedays] magic link for {to}: {url}");
		}
		Err(e) => {
			tracing::error!("[loosedays] failed to send magic link email: {e}");
			tracing::info!("[loosedays] magic link for {to}: {url}");
		}
	}
}
