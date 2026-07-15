import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { requestAccess } from '$lib/server/repo/access';
import { requireUser } from '$lib/server/http';
import type { AccessScope } from '$lib/types';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { scope, start, end } = (await event.request.json()) as {
		scope: AccessScope;
		start: string | null;
		end: string | null;
	};
	requestAccess(user.id, event.params.friendId!, scope, start ?? null, end ?? null);
	return json({ ok: true });
};
