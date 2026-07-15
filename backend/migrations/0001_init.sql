CREATE TABLE users (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	phone_hash text UNIQUE,
	email text NOT NULL UNIQUE,
	display_name text NOT NULL,
	onboarded boolean NOT NULL DEFAULT false,
	created_at timestamptz NOT NULL DEFAULT now()
);

-- Not in the spec's literal sketch, but required by the "sign-up is
-- invite-only" trust model: an allowlist of emails someone already on
-- Loose Days has invited, checked before a magic link is ever issued.
CREATE TABLE invited_emails (
	email text PRIMARY KEY,
	invited_by uuid REFERENCES users(id),
	created_at timestamptz NOT NULL DEFAULT now()
);

-- Named per spec ("login_tokens"), but keyed by email rather than a
-- required user_id fk: sign-in can be the very first time this email is
-- seen (the account is created lazily on first successful magic-link
-- click), so there may be no user row yet when the token is issued.
CREATE TABLE login_tokens (
	token text PRIMARY KEY,
	email text NOT NULL,
	expires_at timestamptz NOT NULL,
	used_at timestamptz,
	created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	expires_at timestamptz NOT NULL,
	created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE friend_edges (
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	friend_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	created_at timestamptz NOT NULL DEFAULT now(),
	PRIMARY KEY (user_id, friend_id)
);

CREATE TABLE calendar_days (
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	date date NOT NULL,
	availability text CHECK (availability IN ('free', 'busy', 'maybe')),
	note text NOT NULL DEFAULT '',
	updated_at timestamptz NOT NULL DEFAULT now(),
	PRIMARY KEY (user_id, date)
);

CREATE TABLE calendar_access_requests (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	requester_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	owner_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	scope text NOT NULL CHECK (scope IN ('range', 'standing')),
	range_start date,
	range_end date,
	detail_level text CHECK (detail_level IN ('full', 'overlap_only')),
	status text NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'approved', 'denied', 'revoked')),
	created_at timestamptz NOT NULL DEFAULT now(),
	responded_at timestamptz
);

CREATE INDEX idx_access_requests_owner ON calendar_access_requests(owner_id, status);
CREATE INDEX idx_access_requests_requester ON calendar_access_requests(requester_id, status);

CREATE TABLE polls (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	creator_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	title text NOT NULL,
	note text NOT NULL DEFAULT '',
	range_start date NOT NULL,
	range_end date NOT NULL,
	created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE poll_invitees (
	id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
	poll_id uuid NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
	user_id uuid REFERENCES users(id) ON DELETE SET NULL,
	-- Hash, not raw digits, per the spec's "no phone numbers stored" rule.
	-- The creator already has the raw number in hand client-side at
	-- creation time to build the SMS/WhatsApp deep link, so nothing is
	-- lost by not persisting it server-side.
	phone_hash text,
	name text NOT NULL,
	access_token text NOT NULL UNIQUE,
	status text NOT NULL DEFAULT 'invited' CHECK (status IN ('invited', 'responded')),
	created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_poll_invitees_poll ON poll_invitees(poll_id);
CREATE INDEX idx_poll_invitees_user ON poll_invitees(user_id);

CREATE TABLE poll_responses (
	poll_id uuid NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
	invitee_id uuid NOT NULL REFERENCES poll_invitees(id) ON DELETE CASCADE,
	date date NOT NULL,
	availability text NOT NULL CHECK (availability IN ('free', 'busy', 'maybe')),
	note text NOT NULL DEFAULT '',
	PRIMARY KEY (poll_id, invitee_id, date)
);
