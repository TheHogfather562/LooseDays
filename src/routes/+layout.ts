import { redirect } from '@sveltejs/kit';
import { db } from '$lib/db.svelte';
import { api } from '$lib/api';
import type { LayoutLoad } from './$types';

// Client-rendered end to end — talks to the real backend under /api instead
// of local/mock state now, but there's still no reason to render on the
// server for a mobile-first single-user-session app like this.
export const ssr = false;

let appDataLoaded = false;

export const load: LayoutLoad = async ({ url }) => {
	const path = url.pathname;

	// Non-user poll response links (SMS/WhatsApp token links) never require auth.
	if (path.startsWith('/p/')) return;

	if (!db.session.checked) {
		await api.loadSession();
	}

	if (db.session.signedIn && db.session.onboarded && !appDataLoaded) {
		appDataLoaded = true;
		await api.bootstrapApp();
	}

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
