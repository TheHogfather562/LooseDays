-- Shareable invite links for number-only invites. Sign-up is invite-only and
-- gated on `invited_emails`, but an SMS invite only knows the recipient's
-- number — never their email — so there'd otherwise be no way for them to land
-- in the allowlist. A link carries an opaque token instead: whoever redeems it
-- at sign-in gets the email they enter allowlisted (recorded in invited_emails,
-- attributed to the inviter), so the normal magic-link flow can proceed.
--
-- Deliberately multi-use within its lifetime: one bulk SMS goes to several
-- people under a single link, so a single-use token would only let the first
-- of them in. The `expires_at` window bounds how long a forwarded link stays
-- live instead.
CREATE TABLE invite_links (
	token text PRIMARY KEY,
	invited_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	expires_at timestamptz NOT NULL,
	created_at timestamptz NOT NULL DEFAULT now()
);
