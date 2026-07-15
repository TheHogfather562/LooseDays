import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { matchContacts } from '$lib/server/repo/friends';
import { requireUser } from '$lib/server/http';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { entries } = (await event.request.json()) as {
		entries?: { name: string; phone: string }[];
	};
	return json(matchContacts(user.id, entries ?? []));
};
