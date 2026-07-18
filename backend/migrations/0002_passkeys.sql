-- Each registered authenticator for a user. `passkey_data` holds the
-- serialized webauthn-rs `Passkey` (public key, counter, backup flags) as a
-- single opaque blob rather than broken-out columns, since the crate treats
-- it as safe-to-persist state and updates it as one unit after each sign-in.
CREATE TABLE passkey_credentials (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	credential_id bytea NOT NULL UNIQUE,
	passkey_data jsonb NOT NULL,
	label text NOT NULL DEFAULT 'Passkey',
	created_at timestamptz NOT NULL DEFAULT now(),
	last_used_at timestamptz
);

CREATE INDEX idx_passkey_credentials_user ON passkey_credentials(user_id);

-- Short-lived server-side state for an in-flight WebAuthn registration or
-- authentication ceremony. The browser only ever holds an opaque challenge
-- id between the start/finish calls (mirroring the login_tokens pattern for
-- magic links) — kept in Postgres rather than in-process memory so this
-- stays correct if the backend is ever scaled to multiple replicas.
CREATE TABLE webauthn_challenges (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	kind text NOT NULL CHECK (kind IN ('registration', 'authentication')),
	user_id uuid REFERENCES users(id) ON DELETE CASCADE,
	state_data jsonb NOT NULL,
	expires_at timestamptz NOT NULL,
	created_at timestamptz NOT NULL DEFAULT now()
);
