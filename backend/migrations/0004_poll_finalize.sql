-- Finalizing a poll records the date (or short range) the group actually
-- settled on, drawn from the best-overlap result. This is a *soft* finalize:
-- responses can still change and the poll can be reopened, so it's an
-- announcement of the decision, not a lock. All three columns are nullable —
-- their absence is what "not finalized yet" means.
--
-- finalized_start / finalized_end mirror the poll's own range columns (a single
-- chosen day has start = end); finalized_at doubles as the "is finalized" flag
-- and records when it happened.
ALTER TABLE polls
	ADD COLUMN finalized_start date,
	ADD COLUMN finalized_end date,
	ADD COLUMN finalized_at timestamptz;
