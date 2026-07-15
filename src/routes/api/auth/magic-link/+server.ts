import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { createMagicLink, isEmailAllowed, normalizeEmail } from '$lib/server/auth';
import { sendMagicLinkEmail } from '$lib/server/email';

export const POST: RequestHandler = async ({ request, url }) => {
	const { email } = (await request.json()) as { email?: string };
	if (!email || !email.includes('@')) return json({ error: 'invalid email' }, { status: 400 });

	// Always respond the same way whether or not the email is allowed, so the
	// invite-only allowlist can't be probed from the sign-in form.
	if (isEmailAllowed(email)) {
		const token = createMagicLink(email);
		const link = `${url.origin}/auth/callback?token=${token}`;
		await sendMagicLinkEmail(normalizeEmail(email), link);
	}
	return json({ ok: true });
};
