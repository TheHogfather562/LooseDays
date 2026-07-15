import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { inviteEmail } from '$lib/server/auth';
import { requireUser } from '$lib/server/http';

// Sign-up is invite-only: any existing user can add an email to the
// allowlist so a friend can request their own magic link.
export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { email } = (await event.request.json()) as { email?: string };
	if (!email || !email.includes('@')) return json({ error: 'invalid email' }, { status: 400 });
	inviteEmail(email, user.id);
	return json({ ok: true });
};
