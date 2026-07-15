import type { Handle } from '@sveltejs/kit';
import { getUserBySession, SESSION_COOKIE } from '$lib/server/auth';

export const handle: Handle = async ({ event, resolve }) => {
	const sessionId = event.cookies.get(SESSION_COOKIE);
	event.locals.user = getUserBySession(sessionId);
	return resolve(event);
};
