import { redirect } from '@sveltejs/kit';
import { db } from '$lib/db.svelte';
import type { LayoutLoad } from './$types';

// Everything runs off local/mock state (see spec.md) — no server to render
// against yet, so this is a client-rendered app end to end.
export const ssr = false;

export const load: LayoutLoad = ({ url }) => {
	const path = url.pathname;
	const isSignin = path === '/signin';
	const isOnboarding = path === '/onboarding';

	if (!db.session.signedIn) {
		if (!isSignin) redirect(303, '/signin');
	} else if (!db.session.onboarded) {
		if (!isOnboarding) redirect(303, '/onboarding');
	} else if (isSignin || isOnboarding || path === '/') {
		redirect(303, '/calendar');
	}
};
