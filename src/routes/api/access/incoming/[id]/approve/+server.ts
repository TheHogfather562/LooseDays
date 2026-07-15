import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { approveRequest } from '$lib/server/repo/access';
import { requireUser } from '$lib/server/http';
import type { DetailLevel } from '$lib/types';

export const POST: RequestHandler = async (event) => {
	const user = requireUser(event);
	const { detailLevel } = (await event.request.json()) as { detailLevel: DetailLevel };
	approveRequest(event.params.id!, user.id, detailLevel);
	return json({ ok: true });
};
