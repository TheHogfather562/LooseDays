import { db } from '../db';
import type { Availability, CalendarDays } from '$lib/types';

export function getCalendarDays(userId: string): CalendarDays {
	const rows = db
		.prepare('SELECT date, availability, note FROM calendar_days WHERE user_id = ?')
		.all(userId) as { date: string; availability: Availability | null; note: string }[];
	const out: CalendarDays = {};
	for (const r of rows) {
		if (r.availability || r.note) out[r.date] = { status: r.availability, note: r.note };
	}
	return out;
}

function upsert(userId: string, date: string, status: Availability | null, note: string) {
	if (!status && !note) {
		db.prepare('DELETE FROM calendar_days WHERE user_id = ? AND date = ?').run(userId, date);
		return;
	}
	db.prepare(
		`INSERT INTO calendar_days (user_id, date, availability, note, updated_at)
		 VALUES (?, ?, ?, ?, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
		 ON CONFLICT(user_id, date) DO UPDATE SET
		   availability = excluded.availability,
		   note = excluded.note,
		   updated_at = excluded.updated_at`
	).run(userId, date, status, note);
}

export function setDayStatus(userId: string, date: string, status: Availability | null): void {
	const row = db
		.prepare('SELECT note FROM calendar_days WHERE user_id = ? AND date = ?')
		.get(userId, date) as { note: string } | undefined;
	upsert(userId, date, status, row?.note ?? '');
}

export function setDayNote(userId: string, date: string, note: string): void {
	const row = db
		.prepare('SELECT availability FROM calendar_days WHERE user_id = ? AND date = ?')
		.get(userId, date) as { availability: Availability | null } | undefined;
	upsert(userId, date, row?.availability ?? null, note);
}

export function clearDay(userId: string, date: string): void {
	db.prepare('DELETE FROM calendar_days WHERE user_id = ? AND date = ?').run(userId, date);
}
