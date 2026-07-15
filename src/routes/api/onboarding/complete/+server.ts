import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { completeOnboarding } from '$lib/server/auth';
import { requireUser } from '$lib/server/http';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	completeOnboarding(user.id);
	return json({ ok: true });
};
