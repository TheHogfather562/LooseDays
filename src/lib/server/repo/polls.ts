import { randomBytes, randomUUID } from 'node:crypto';
import { db } from '../db';
import type { Availability, Poll, PollInvitee } from '$lib/types';

interface PollRow {
	id: string;
	creator_id: string;
	title: string;
	note: string;
	range_start: string;
	range_end: string;
}

interface InviteeRow {
	id: string;
	poll_id: string;
	user_id: string | null;
	phone: string | null;
	name: string;
	access_token: string;
	status: 'invited' | 'responded';
}

function newToken(): string {
	return randomBytes(9).toString('base64url');
}

function buildPoll(pollRow: PollRow, viewerInviteeId?: string): Poll {
	const invitees = db
		.prepare('SELECT * FROM poll_invitees WHERE poll_id = ? ORDER BY rowid')
		.all(pollRow.id) as InviteeRow[];
	const responseRows = db
		.prepare('SELECT invitee_id, date, availability FROM poll_responses WHERE poll_id = ?')
		.all(pollRow.id) as { invitee_id: string; date: string; availability: Availability }[];

	const responses: Poll['responses'] = {};
	for (const r of responseRows) {
		(responses[r.invitee_id] ??= {})[r.date] = r.availability;
	}

	const dtoInvitees: PollInvitee[] = invitees.map((inv) => ({
		id: inv.id,
		userId: inv.user_id,
		phone: inv.phone ?? undefined,
		name: inv.name,
		status: inv.status,
		accessToken: inv.access_token,
		isMe: inv.id === viewerInviteeId
	}));

	return {
		id: pollRow.id,
		title: pollRow.title,
		note: pollRow.note,
		creatorId: pollRow.creator_id,
		rangeStart: pollRow.range_start,
		rangeEnd: pollRow.range_end,
		invitees: dtoInvitees,
		responses
	};
}

export function createPoll(
	creatorId: string,
	input: {
		title: string;
		note: string;
		start: string;
		end: string;
		friendIds: string[];
		phoneChips: string[];
	}
): Poll {
	const pollId = randomUUID();
	db.prepare(
		`INSERT INTO polls (id, creator_id, title, note, range_start, range_end)
		 VALUES (?, ?, ?, ?, ?, ?)`
	).run(pollId, creatorId, input.title || 'Untitled poll', input.note, input.start, input.end);

	const insertInvitee = db.prepare(
		`INSERT INTO poll_invitees (id, poll_id, user_id, phone, name, access_token, status)
		 VALUES (?, ?, ?, ?, ?, ?, 'invited')`
	);

	const creator = db.prepare('SELECT display_name FROM users WHERE id = ?').get(creatorId) as {
		display_name: string;
	};
	insertInvitee.run(randomUUID(), pollId, creatorId, null, creator.display_name, newToken());

	const friends =
		input.friendIds.length > 0
			? (db
					.prepare(
						`SELECT id, display_name FROM users WHERE id IN (${input.friendIds.map(() => '?').join(',')})`
					)
					.all(...input.friendIds) as { id: string; display_name: string }[])
			: [];
	for (const f of friends) {
		insertInvitee.run(randomUUID(), pollId, f.id, null, f.display_name, newToken());
	}

	for (const phone of input.phoneChips) {
		insertInvitee.run(randomUUID(), pollId, null, phone, phone, newToken());
	}

	const pollRow = db.prepare('SELECT * FROM polls WHERE id = ?').get(pollId) as PollRow;
	const myInvitee = db
		.prepare('SELECT id FROM poll_invitees WHERE poll_id = ? AND user_id = ?')
		.get(pollId, creatorId) as { id: string };
	return buildPoll(pollRow, myInvitee.id);
}

export function listPollsForUser(userId: string): Poll[] {
	const rows = db
		.prepare(
			`SELECT DISTINCT p.* FROM polls p
			 LEFT JOIN poll_invitees pi ON pi.poll_id = p.id
			 WHERE p.creator_id = ? OR pi.user_id = ?
			 ORDER BY p.created_at DESC`
		)
		.all(userId, userId) as PollRow[];
	return rows.map((row) => {
		const mine = db
			.prepare('SELECT id FROM poll_invitees WHERE poll_id = ? AND user_id = ?')
			.get(row.id, userId) as { id: string } | undefined;
		return buildPoll(row, mine?.id);
	});
}

export function getPollForUser(pollId: string, userId: string): Poll | null {
	const row = db.prepare('SELECT * FROM polls WHERE id = ?').get(pollId) as PollRow | undefined;
	if (!row) return null;
	const mine = db
		.prepare('SELECT id FROM poll_invitees WHERE poll_id = ? AND user_id = ?')
		.get(pollId, userId) as { id: string } | undefined;
	if (row.creator_id !== userId && !mine) return null;
	return buildPoll(row, mine?.id);
}

function saveResponses(pollId: string, inviteeId: string, responses: Record<string, Availability>) {
	const del = db.prepare('DELETE FROM poll_responses WHERE poll_id = ? AND invitee_id = ?');
	const insert = db.prepare(
		'INSERT INTO poll_responses (poll_id, invitee_id, date, availability) VALUES (?, ?, ?, ?)'
	);
	const tx = db.transaction((entries: [string, Availability][]) => {
		del.run(pollId, inviteeId);
		for (const [date, availability] of entries) insert.run(pollId, inviteeId, date, availability);
		db.prepare("UPDATE poll_invitees SET status = 'responded' WHERE id = ?").run(inviteeId);
	});
	tx(Object.entries(responses) as [string, Availability][]);
}

export function submitResponse(
	pollId: string,
	userId: string,
	responses: Record<string, Availability>
): boolean {
	const invitee = db
		.prepare('SELECT id FROM poll_invitees WHERE poll_id = ? AND user_id = ?')
		.get(pollId, userId) as { id: string } | undefined;
	if (!invitee) return false;
	saveResponses(pollId, invitee.id, responses);
	return true;
}

export function getPollByToken(token: string): Poll | null {
	const invitee = db
		.prepare('SELECT poll_id FROM poll_invitees WHERE access_token = ?')
		.get(token) as { poll_id: string } | undefined;
	if (!invitee) return null;
	const row = db.prepare('SELECT * FROM polls WHERE id = ?').get(invitee.poll_id) as
		| PollRow
		| undefined;
	if (!row) return null;
	const mine = db.prepare('SELECT id FROM poll_invitees WHERE access_token = ?').get(token) as {
		id: string;
	};
	return buildPoll(row, mine.id);
}

export function submitResponseByToken(
	token: string,
	responses: Record<string, Availability>
): boolean {
	const invitee = db
		.prepare('SELECT id, poll_id FROM poll_invitees WHERE access_token = ?')
		.get(token) as { id: string; poll_id: string } | undefined;
	if (!invitee) return false;
	saveResponses(invitee.poll_id, invitee.id, responses);
	return true;
}
