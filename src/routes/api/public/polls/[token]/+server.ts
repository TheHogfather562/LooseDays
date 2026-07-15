import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getPollByToken } from '$lib/server/repo/polls';

// Unauthenticated on purpose — this is the link a non-user invitee gets over
// SMS/WhatsApp, per the spec's "works for non-users via a token link too".
export const GET: RequestHandler = async ({ params }) => {
	const poll = getPollByToken(params.token!);
	if (!poll) error(404, 'Poll not found');
	return json(poll);
};
