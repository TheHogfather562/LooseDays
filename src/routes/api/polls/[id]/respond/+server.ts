import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { submitResponse } from '$lib/server/repo/polls';
import { requireUser } from '$lib/server/http';
import type { Availability } from '$lib/types';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { responses } = (await event.request.json()) as {
		responses: Record<string, Availability>;
	};
	const ok = submitResponse(event.params.id!, user.id, responses);
	if (!ok) error(403, 'Not an invitee on this poll');
	return json({ ok: true });
};
