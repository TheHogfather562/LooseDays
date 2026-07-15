import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { setUserPhone } from '$lib/server/auth';
import { requireUser } from '$lib/server/http';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { phone } = (await event.request.json()) as { phone?: string };
	if (!phone?.trim()) return json({ error: 'phone required' }, { status: 400 });
	setUserPhone(user.id, phone);
	return json({ ok: true });
};
