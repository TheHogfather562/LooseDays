import { error, type RequestEvent } from '@sveltejs/kit';
import type { SessionUser } from './auth';

export function requireUser(event: RequestEvent): SessionUser {
	if (!event.locals.user) error(401, 'Sign in required');
	return event.locals.user;
}
