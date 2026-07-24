-- Records that a user has been sent the Sunday "plan next week" reminder for a
-- given week, so the scheduler is idempotent: a process restart around the
-- send time (or a startup catch-up run) never double-mails anyone. Keyed by
-- the Monday the reminded week starts on.
CREATE TABLE weekly_nudges (
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	week_start date NOT NULL,
	sent_at timestamptz NOT NULL DEFAULT now(),
	PRIMARY KEY (user_id, week_start)
);
