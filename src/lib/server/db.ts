// Real persistent store for the app — SQLite via better-sqlite3. Chosen over
// hosted Postgres so this MVP can be self-hosted with zero external services;
// the table shapes below still mirror the Postgres schema in the spec.

import DatabaseConstructor from 'better-sqlite3';
import { mkdirSync } from 'node:fs';
import { dirname } from 'node:path';
import { env } from '$env/dynamic/private';

type Database = InstanceType<typeof DatabaseConstructor>;

const DATABASE_PATH = env.DATABASE_PATH || './data/loosedays.db';

function open(): Database {
	if (DATABASE_PATH !== ':memory:') mkdirSync(dirname(DATABASE_PATH), { recursive: true });
	const database = new DatabaseConstructor(DATABASE_PATH);
	database.pragma('journal_mode = WAL');
	database.pragma('foreign_keys = ON');
	return database;
}

const globalForDb = globalThis as unknown as { __loosedaysDb?: Database };

export const db: Database = globalForDb.__loosedaysDb ?? open();
globalForDb.__loosedaysDb = db;

db.exec(`
	CREATE TABLE IF NOT EXISTS users (
		id TEXT PRIMARY KEY,
		email TEXT NOT NULL UNIQUE,
		phone TEXT,
		display_name TEXT NOT NULL,
		onboarded INTEGER NOT NULL DEFAULT 0,
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
	);

	CREATE TABLE IF NOT EXISTS invited_emails (
		email TEXT PRIMARY KEY,
		invited_by TEXT,
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
	);

	CREATE TABLE IF NOT EXISTS magic_links (
		token TEXT PRIMARY KEY,
		email TEXT NOT NULL,
		expires_at TEXT NOT NULL,
		used INTEGER NOT NULL DEFAULT 0,
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
	);

	CREATE TABLE IF NOT EXISTS sessions (
		id TEXT PRIMARY KEY,
		user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		expires_at TEXT NOT NULL,
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
	);

	CREATE TABLE IF NOT EXISTS friend_edges (
		user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		friend_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
		PRIMARY KEY (user_id, friend_id)
	);

	CREATE TABLE IF NOT EXISTS calendar_days (
		user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		date TEXT NOT NULL,
		availability TEXT,
		note TEXT NOT NULL DEFAULT '',
		updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
		PRIMARY KEY (user_id, date)
	);

	CREATE TABLE IF NOT EXISTS calendar_access_requests (
		id TEXT PRIMARY KEY,
		requester_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		owner_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		scope TEXT NOT NULL,
		range_start TEXT,
		range_end TEXT,
		detail_level TEXT,
		status TEXT NOT NULL DEFAULT 'pending',
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
		responded_at TEXT
	);

	CREATE TABLE IF NOT EXISTS polls (
		id TEXT PRIMARY KEY,
		creator_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
		title TEXT NOT NULL,
		note TEXT NOT NULL DEFAULT '',
		range_start TEXT NOT NULL,
		range_end TEXT NOT NULL,
		created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
	);

	CREATE TABLE IF NOT EXISTS poll_invitees (
		id TEXT PRIMARY KEY,
		poll_id TEXT NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
		user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
		phone TEXT,
		name TEXT NOT NULL,
		access_token TEXT NOT NULL UNIQUE,
		status TEXT NOT NULL DEFAULT 'invited'
	);

	CREATE TABLE IF NOT EXISTS poll_responses (
		poll_id TEXT NOT NULL REFERENCES polls(id) ON DELETE CASCADE,
		invitee_id TEXT NOT NULL REFERENCES poll_invitees(id) ON DELETE CASCADE,
		date TEXT NOT NULL,
		availability TEXT NOT NULL,
		note TEXT NOT NULL DEFAULT '',
		PRIMARY KEY (poll_id, invitee_id, date)
	);

	CREATE INDEX IF NOT EXISTS idx_access_requests_owner ON calendar_access_requests(owner_id, status);
	CREATE INDEX IF NOT EXISTS idx_access_requests_requester ON calendar_access_requests(requester_id, status);
	CREATE INDEX IF NOT EXISTS idx_poll_invitees_poll ON poll_invitees(poll_id);
	CREATE INDEX IF NOT EXISTS idx_poll_invitees_user ON poll_invitees(user_id);
`);

// Bootstrap the founding account so there's at least one invited email to sign
// in with on a fresh database.
const adminEmail = env.ADMIN_EMAIL?.trim().toLowerCase();
if (adminEmail) {
	db.prepare('INSERT OR IGNORE INTO invited_emails (email, invited_by) VALUES (?, NULL)').run(
		adminEmail
	);
}
