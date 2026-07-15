import { randomUUID } from 'node:crypto';
import { db } from '../db';
import type {
	AccessScope,
	Availability,
	DetailLevel,
	FriendAccessGrant,
	IncomingRequest,
	StandingAccessGrant
} from '$lib/types';

interface AccessRequestRow {
	id: string;
	requester_id: string;
	owner_id: string;
	scope: AccessScope;
	range_start: string | null;
	range_end: string | null;
	detail_level: DetailLevel | null;
	status: 'pending' | 'approved' | 'denied' | 'revoked';
	responded_at: string | null;
}

export function requestAccess(
	requesterId: string,
	ownerId: string,
	scope: AccessScope,
	rangeStart: string | null,
	rangeEnd: string | null
): void {
	db.prepare(
		`INSERT INTO calendar_access_requests (id, requester_id, owner_id, scope, range_start, range_end, status)
		 VALUES (?, ?, ?, ?, ?, ?, 'pending')`
	).run(randomUUID(), requesterId, ownerId, scope, rangeStart, rangeEnd);
}

export function getIncomingRequests(ownerId: string): IncomingRequest[] {
	const rows = db
		.prepare(
			`SELECT r.id, r.requester_id, u.display_name AS requester_name, r.scope, r.range_start, r.range_end
			 FROM calendar_access_requests r
			 JOIN users u ON u.id = r.requester_id
			 WHERE r.owner_id = ? AND r.status = 'pending'
			 ORDER BY r.created_at`
		)
		.all(ownerId) as {
		id: string;
		requester_id: string;
		requester_name: string;
		scope: AccessScope;
		range_start: string | null;
		range_end: string | null;
	}[];
	return rows.map((r) => ({
		id: r.id,
		requesterId: r.requester_id,
		requesterName: r.requester_name,
		scope: r.scope,
		rangeStart: r.range_start,
		rangeEnd: r.range_end
	}));
}

export function approveRequest(requestId: string, ownerId: string, detailLevel: DetailLevel): void {
	db.prepare(
		`UPDATE calendar_access_requests
		 SET status = 'approved', detail_level = ?, responded_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
		 WHERE id = ? AND owner_id = ? AND status = 'pending'`
	).run(detailLevel, requestId, ownerId);
}

export function denyRequest(requestId: string, ownerId: string): void {
	db.prepare(
		`UPDATE calendar_access_requests
		 SET status = 'denied', responded_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
		 WHERE id = ? AND owner_id = ? AND status = 'pending'`
	).run(requestId, ownerId);
}

export function revokeAccess(requestId: string, ownerId: string): void {
	db.prepare(
		`UPDATE calendar_access_requests
		 SET status = 'revoked', responded_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
		 WHERE id = ? AND owner_id = ? AND status = 'approved'`
	).run(requestId, ownerId);
}

/** Access I (the requester) have been granted into friends' calendars, keyed
 * by friend id. When more than one grant exists for the same friend, the most
 * recently-approved one wins. */
export function getMyAccess(requesterId: string): Record<string, FriendAccessGrant> {
	const rows = db
		.prepare(
			`SELECT * FROM calendar_access_requests
			 WHERE requester_id = ? AND status = 'approved'
			 ORDER BY responded_at ASC`
		)
		.all(requesterId) as AccessRequestRow[];
	const out: Record<string, FriendAccessGrant> = {};
	for (const r of rows) {
		out[r.owner_id] = {
			level: r.detail_level === 'full' ? 'full' : 'overlap',
			scope: r.scope,
			rangeStart: r.range_start ?? undefined,
			rangeEnd: r.range_end ?? undefined
		};
	}
	return out;
}

export function getOutgoingPending(requesterId: string): Record<string, boolean> {
	const rows = db
		.prepare(
			`SELECT DISTINCT owner_id FROM calendar_access_requests WHERE requester_id = ? AND status = 'pending'`
		)
		.all(requesterId) as { owner_id: string }[];
	const out: Record<string, boolean> = {};
	for (const r of rows) out[r.owner_id] = true;
	return out;
}

/** Everyone who currently has approved access into my calendar. */
export function getStandingAccess(ownerId: string): StandingAccessGrant[] {
	const rows = db
		.prepare(
			`SELECT r.id, r.requester_id, u.display_name AS requester_name, r.detail_level
			 FROM calendar_access_requests r
			 JOIN users u ON u.id = r.requester_id
			 WHERE r.owner_id = ? AND r.status = 'approved'
			 ORDER BY r.responded_at DESC`
		)
		.all(ownerId) as {
		id: string;
		requester_id: string;
		requester_name: string;
		detail_level: DetailLevel;
	}[];
	return rows.map((r) => ({
		id: r.id,
		friendId: r.requester_id,
		friendName: r.requester_name,
		detailLevel: r.detail_level
	}));
}

/** What `viewerId` is allowed to see of `ownerId`'s calendar, already filtered
 * to the granted date range and detail level — full status/notes, or (in
 * overlap mode) only the dates the owner marked free, with no other detail
 * leaked. Returns {} if there's no active grant. */
export function getVisibleFriendCalendar(
	viewerId: string,
	ownerId: string
): Record<string, Availability> {
	const grant = db
		.prepare(
			`SELECT * FROM calendar_access_requests
			 WHERE requester_id = ? AND owner_id = ? AND status = 'approved'
			 ORDER BY responded_at DESC LIMIT 1`
		)
		.get(viewerId, ownerId) as AccessRequestRow | undefined;
	if (!grant) return {};

	const rows = db
		.prepare(
			'SELECT date, availability FROM calendar_days WHERE user_id = ? AND availability IS NOT NULL'
		)
		.all(ownerId) as { date: string; availability: Availability }[];

	const inRange = (date: string) =>
		grant.scope !== 'range' ||
		((!grant.range_start || date >= grant.range_start) && (!grant.range_end || date <= grant.range_end));

	const out: Record<string, Availability> = {};
	for (const r of rows) {
		if (!inRange(r.date)) continue;
		if (grant.detail_level === 'full') out[r.date] = r.availability;
		else if (r.availability === 'free') out[r.date] = 'free';
	}
	return out;
}
