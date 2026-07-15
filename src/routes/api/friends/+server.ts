import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { addFriend, getFriends } from '$lib/server/repo/friends';
import { requireUser } from '$lib/server/http';

export const GET: RequestHandler = async (event) => {
	const user = requireUser(event);
	return json(getFriends(user.id));
};

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { userId } = (await event.request.json()) as { userId?: string };
	if (!userId) return json({ error: 'userId required' }, { status: 400 });
	addFriend(user.id, userId);
	return json({ ok: true });
};
