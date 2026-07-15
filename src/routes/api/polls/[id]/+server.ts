import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getPollForUser } from '$lib/server/repo/polls';
import { requireUser } from '$lib/server/http';

export const GET: RequestHandler = async (event) => {
	const user = requireUser(event);
	const poll = getPollForUser(event.params.id!, user.id);
	if (!poll) error(404, 'Poll not found');
	return json(poll);
};
