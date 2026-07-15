// Reactive client-side cache of server state. `api.ts` is the only thing that
// writes into this — components read from `db` and call `api.*` to mutate.
// There's no local persistence anymore: the SQLite backend is the source of
// truth, and this cache is (re)populated by `bootstrap()`/`api` calls.

import type {
	CalendarDays,
	FriendAccessGrant,
	IncomingRequest,
	Poll,
	StandingAccessGrant,
	User
} from './types';

interface AppState {
	session: { checked: boolean; signedIn: boolean; onboarded: boolean };
	currentUser: User | null;
	calStatuses: CalendarDays;
	friendAccess: Record<string, FriendAccessGrant>;
	outgoingPending: Record<string, boolean>;
	incomingRequests: IncomingRequest[];
	standingAccess: StandingAccessGrant[];
	polls: Poll[];
}

function empty(): AppState {
	return {
		session: { checked: false, signedIn: false, onboarded: false },
		currentUser: null,
		calStatuses: {},
		friendAccess: {},
		outgoingPending: {},
		incomingRequests: [],
		standingAccess: [],
		polls: []
	};
}

export const db = $state(empty());

export function resetDb() {
	Object.assign(db, empty());
}
