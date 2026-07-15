import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { denyRequest } from '$lib/server/repo/access';
import { requireUser } from '$lib/server/http';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	denyRequest(event.params.id!, user.id);
	return json({ ok: true });
};
