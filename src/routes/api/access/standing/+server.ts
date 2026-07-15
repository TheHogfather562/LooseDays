import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getStandingAccess } from '$lib/server/repo/access';
import { requireUser } from '$lib/server/http';

export const GET: RequestHandler = async (event) => {
	const user = requireUser(event);
	return json(getStandingAccess(user.id));
};
