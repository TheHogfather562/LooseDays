import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getCalendarDays } from '$lib/server/repo/calendar';
import { requireUser } from '$lib/server/http';

export const GET: RequestHandler = async (event) => {
	const user = requireUser(event);
	return json(getCalendarDays(user.id));
};
