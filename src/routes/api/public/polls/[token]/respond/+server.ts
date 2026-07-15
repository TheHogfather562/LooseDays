import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { submitResponseByToken } from '$lib/server/repo/polls';
import type { Availability } from '$lib/types';

export const POST: RequestHandler = async ({ params, request }) => {
	const { responses } = (await request.json()) as { responses: Record<string, Availability> };
	const ok = submitResponseByToken(params.token!, responses);
	if (!ok) error(404, 'Poll not found');
	return json({ ok: true });
};
