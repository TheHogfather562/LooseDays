import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { revokeAccess } from '$lib/server/repo/access';
import { requireUser } from '$lib/server/http';

export const DELETE: RequestHandler = async (event) => {
	const user = requireUser(event);
	revokeAccess(event.params.id!, user.id);
	return json({ ok: true });
};
