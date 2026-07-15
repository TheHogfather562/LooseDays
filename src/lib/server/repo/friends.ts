import { db } from '../db';
import type { Contact, Friend } from '$lib/types';

/** Digits-only, US-leading-1-stripped so "+1 415-555-0101" and "415-555-0101"
 * both match the same underlying number. Good enough for a friends-only MVP. */
export function normalizePhone(phone: string): string {
	const digits = phone.replace(/\D/g, '');
	return digits.length === 11 && digits.startsWith('1') ? digits.slice(1) : digits;
}

export function getFriends(userId: string): Friend[] {
	const rows = db
		.prepare(
			`SELECT u.id, u.display_name, u.phone FROM friend_edges fe
			 JOIN users u ON u.id = fe.friend_id
			 WHERE fe.user_id = ?
			 ORDER BY u.display_name`
		)
		.all(userId) as { id: string; display_name: string; phone: string | null }[];
	return rows.map((r) => ({ id: r.id, displayName: r.display_name, phone: r.phone ?? '' }));
}

function addFriendEdge(userId: string, friendId: string) {
	const insert = db.prepare(
		'INSERT OR IGNORE INTO friend_edges (user_id, friend_id) VALUES (?, ?)'
	);
	insert.run(userId, friendId);
	insert.run(friendId, userId);
}

/** Matches raw phone numbers (e.g. pasted from a contacts app) against
 * existing accounts, without persisting anything — nothing here is a real
 * "contacts" table since the spec doesn't have one either. */
export function matchContacts(
	userId: string,
	entries: { name: string; phone: string }[]
): Contact[] {
	const users = db
		.prepare('SELECT id, phone, display_name FROM users WHERE phone IS NOT NULL')
		.all() as { id: string; phone: string; display_name: string }[];
	const byNormalizedPhone = new Map(users.map((u) => [normalizePhone(u.phone), u]));
	const existingFriends = new Set(
		(
			db.prepare('SELECT friend_id FROM friend_edges WHERE user_id = ?').all(userId) as {
				friend_id: string;
			}[]
		).map((r) => r.friend_id)
	);

	return entries.map((entry, i) => {
		const match = byNormalizedPhone.get(normalizePhone(entry.phone));
		const matched = !!match && match.id !== userId;
		return {
			id: `c${i}_${normalizePhone(entry.phone)}`,
			name: entry.name || entry.phone,
			phone: entry.phone,
			matched,
			userId: matched ? match!.id : null,
			alreadyFriend: matched && existingFriends.has(match!.id)
		} as Contact & { alreadyFriend: boolean };
	});
}

export function addFriend(userId: string, friendUserId: string): void {
	if (userId === friendUserId) return;
	const exists = db.prepare('SELECT 1 FROM users WHERE id = ?').get(friendUserId);
	if (!exists) return;
	addFriendEdge(userId, friendUserId);
}
