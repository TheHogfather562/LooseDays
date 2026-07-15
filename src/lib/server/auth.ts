import { randomBytes, randomUUID } from 'node:crypto';
import { db } from './db';

export const SESSION_COOKIE = 'ld_session';

const SESSION_TTL_MS = 1000 * 60 * 60 * 24 * 30; // 30 days
const MAGIC_LINK_TTL_MS = 1000 * 60 * 15; // 15 minutes

export interface SessionUser {
	id: string;
	email: string;
	displayName: string;
	phone: string | null;
	onboarded: boolean;
}

interface UserRow {
	id: string;
	email: string;
	phone: string | null;
	display_name: string;
	onboarded: number;
}

function toSessionUser(row: UserRow): SessionUser {
	return {
		id: row.id,
		email: row.email,
		displayName: row.display_name,
		phone: row.phone,
		onboarded: !!row.onboarded
	};
}

function newToken(): string {
	return randomBytes(32).toString('hex');
}

export function normalizeEmail(email: string): string {
	return email.trim().toLowerCase();
}

/** Sign-up is invite-only: allow a magic link only for existing users or
 * emails someone already on Loose Days has invited. */
export function isEmailAllowed(email: string): boolean {
	const e = normalizeEmail(email);
	if (db.prepare('SELECT 1 FROM users WHERE email = ?').get(e)) return true;
	return !!db.prepare('SELECT 1 FROM invited_emails WHERE email = ?').get(e);
}

export function inviteEmail(email: string, invitedBy: string): void {
	const e = normalizeEmail(email);
	db.prepare('INSERT OR IGNORE INTO invited_emails (email, invited_by) VALUES (?, ?)').run(
		e,
		invitedBy
	);
}

export function createMagicLink(email: string): string {
	const token = newToken();
	const expiresAt = new Date(Date.now() + MAGIC_LINK_TTL_MS).toISOString();
	db.prepare('INSERT INTO magic_links (token, email, expires_at) VALUES (?, ?, ?)').run(
		token,
		normalizeEmail(email),
		expiresAt
	);
	return token;
}

/** Verifies + burns a magic-link token, returning the email it was issued to. */
export function consumeMagicLink(token: string): string | null {
	const row = db.prepare('SELECT * FROM magic_links WHERE token = ?').get(token) as
		| { token: string; email: string; expires_at: string; used: number }
		| undefined;
	if (!row || row.used || new Date(row.expires_at).getTime() < Date.now()) return null;
	db.prepare('UPDATE magic_links SET used = 1 WHERE token = ?').run(token);
	return row.email;
}

export function findOrCreateUserByEmail(email: string): SessionUser {
	const e = normalizeEmail(email);
	const existing = db.prepare('SELECT * FROM users WHERE email = ?').get(e) as
		| UserRow
		| undefined;
	if (existing) return toSessionUser(existing);

	const id = randomUUID();
	const displayName = e.split('@')[0];
	db.prepare('INSERT INTO users (id, email, display_name) VALUES (?, ?, ?)').run(
		id,
		e,
		displayName
	);
	return { id, email: e, displayName, phone: null, onboarded: false };
}

export function createSession(userId: string): { id: string; expiresAt: Date } {
	const id = randomUUID();
	const expiresAt = new Date(Date.now() + SESSION_TTL_MS);
	db.prepare('INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)').run(
		id,
		userId,
		expiresAt.toISOString()
	);
	return { id, expiresAt };
}

export function destroySession(sessionId: string): void {
	db.prepare('DELETE FROM sessions WHERE id = ?').run(sessionId);
}

export function getUserBySession(sessionId: string | undefined): SessionUser | null {
	if (!sessionId) return null;
	const row = db
		.prepare(
			`SELECT u.* FROM sessions s JOIN users u ON u.id = s.user_id
			 WHERE s.id = ? AND s.expires_at > ?`
		)
		.get(sessionId, new Date().toISOString()) as UserRow | undefined;
	return row ? toSessionUser(row) : null;
}

export function setUserPhone(userId: string, phone: string): void {
	db.prepare('UPDATE users SET phone = ? WHERE id = ?').run(phone.trim(), userId);
}

export function completeOnboarding(userId: string): void {
	db.prepare('UPDATE users SET onboarded = 1 WHERE id = ?').run(userId);
}
