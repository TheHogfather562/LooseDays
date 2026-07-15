import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { clearDay, setDayNote, setDayStatus } from '$lib/server/repo/calendar';
import { requireUser } from '$lib/server/http';
import type { Availability } from '$lib/types';

export const PATCH: RequestHandler = async (event) => {
	const user = requireUser(event);
	const date = event.params.date!;
	const body = (await event.request.json()) as { status?: Availability | null; note?: string };
	if ('status' in body) setDayStatus(user.id, date, body.status ?? null);
	if ('note' in body) setDayNote(user.id, date, body.note ?? '');
	return json({ ok: true });
};

export const DELETE: RequestHandler = async (event) => {
	const user = requireUser(event);
	clearDay(user.id, event.params.date!);
	return json({ ok: true });
};
