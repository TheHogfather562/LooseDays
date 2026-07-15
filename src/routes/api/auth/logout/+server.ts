import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { destroySession, SESSION_COOKIE } from '$lib/server/auth';

export const POST: RequestHandler = async ({ cookies }) => {
	const sessionId = cookies.get(SESSION_COOKIE);
	if (sessionId) destroySession(sessionId);
	cookies.delete(SESSION_COOKIE, { path: '/' });
	return json({ ok: true });
};
