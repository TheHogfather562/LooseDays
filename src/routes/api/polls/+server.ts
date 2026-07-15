import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { createPoll, listPollsForUser } from '$lib/server/repo/polls';
import { requireUser } from '$lib/server/http';

export const GET: RequestHandler = async (event) => {
	const user = requireUser(event);
	return json(listPollsForUser(user.id));
};

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const input = (await event.request.json()) as {
		title: string;
		note: string;
		start: string;
		end: string;
		friendIds: string[];
		phoneChips: string[];
	};
	return json(createPoll(user.id, input));
};
