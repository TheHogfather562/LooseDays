use std::sync::Arc;

use url::Url;
use webauthn_rs::prelude::{Webauthn, WebauthnBuilder};

use crate::config::Config;

/// The WebAuthn RP ID and origin must exactly match the domain the browser
/// sees, so both are derived from `PUBLIC_ORIGIN` rather than configured
/// separately — the same value the magic-link flow already requires to match.
pub fn build(config: &Config) -> anyhow::Result<Arc<Webauthn>> {
	let origin = Url::parse(&config.public_origin)?;
	let rp_id = origin
		.host_str()
		.ok_or_else(|| anyhow::anyhow!("PUBLIC_ORIGIN must include a host to use as the WebAuthn RP ID"))?
		.to_string();
	let webauthn = WebauthnBuilder::new(&rp_id, &origin)?
		.rp_name(&config.rp_name)
		.build()?;
	Ok(Arc::new(webauthn))
}
