// Local reactive "database" — the client-side cache that a real API layer
// would populate. Everything mutable in the app lives here and is persisted
// to localStorage so state survives a reload. `api.ts` is the only thing that
// reads/writes this module; components go through `api.ts`.

import { browser } from '$app/environment';
import {
	friendAccessOverrides,
	initialCalStatuses,
	initialIncomingRequests,
	initialPolls,
	initialStandingAccess
} from './mock-data';
import type {
	CalendarDays,
	FriendAccessGrant,
	IncomingRequest,
	Poll,
	StandingAccessGrant
} from './types';

const STORAGE_KEY = 'loosedays.db.v1';

interface PersistedState {
	session: { signedIn: boolean; onboarded: boolean };
	calStatuses: CalendarDays;
	friendAccess: Record<string, FriendAccessGrant>;
	outgoingPending: Record<string, boolean>;
	onboardingAdded: Record<string, boolean>;
	incomingRequests: IncomingRequest[];
	standingAccess: StandingAccessGrant[];
	polls: Poll[];
}

function seed(): PersistedState {
	return {
		session: { signedIn: false, onboarded: false },
		calStatuses: structuredClone(initialCalStatuses),
		friendAccess: structuredClone(friendAccessOverrides),
		outgoingPending: {},
		onboardingAdded: {},
		incomingRequests: structuredClone(initialIncomingRequests),
		standingAccess: structuredClone(initialStandingAccess),
		polls: structuredClone(initialPolls)
	};
}

function loadPersisted(): PersistedState {
	if (browser) {
		try {
			const raw = localStorage.getItem(STORAGE_KEY);
			if (raw) return { ...seed(), ...JSON.parse(raw) };
		} catch {
			// fall through to fresh seed
		}
	}
	return seed();
}

export const db = $state(loadPersisted());

export function persist() {
	if (!browser) return;
	localStorage.setItem(STORAGE_KEY, JSON.stringify($state.snapshot(db)));
}

export function resetDb() {
	Object.assign(db, seed());
	persist();
}
