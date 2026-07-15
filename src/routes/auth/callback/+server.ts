import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import {
	consumeMagicLink,
	createSession,
	findOrCreateUserByEmail,
	SESSION_COOKIE
} from '$lib/server/auth';

export const GET: RequestHandler = async ({ url, cookies }) => {
	const token = url.searchParams.get('token');
	const email = token ? consumeMagicLink(token) : null;
	if (!email) redirect(303, '/signin?error=expired');

	const user = findOrCreateUserByEmail(email);
	const session = createSession(user.id);
	cookies.set(SESSION_COOKIE, session.id, {
		path: '/',
		httpOnly: true,
		secure: url.protocol === 'https:',
		sameSite: 'lax',
		expires: session.expiresAt
	});

	redirect(303, user.onboarded ? '/calendar' : '/onboarding');
};
