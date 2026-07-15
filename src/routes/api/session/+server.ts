import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ locals }) => {
	const user = locals.user;
	return json({
		signedIn: !!user,
		onboarded: !!user?.onboarded,
		user: user
			? { id: user.id, displayName: user.displayName, email: user.email, phone: user.phone }
			: null
	});
};
