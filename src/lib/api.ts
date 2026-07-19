// Thin wrapper around the real backend (see src/routes/api/**). Components
// should always go through this module rather than talking to fetch directly
// — it's also the only thing that writes into the `db` client-side cache.

import { db } from './db.svelte';
import { rememberPhone } from './phonebook';
import type {
	AccessScope,
	Availability,
	Contact,
	DetailLevel,
	EmailSearchResult,
	Friend,
	Passkey,
	Poll
} from './types';
import type { CreationOptionsJSON, RequestOptionsJSON } from './webauthn';

async function req<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(path, {
		...init,
		headers: { 'Content-Type': 'application/json', ...(init?.headers ?? {}) }
	});
	if (!res.ok) {
		const body = await res.json().catch(() => null);
		throw new Error(body?.message || `${init?.method ?? 'GET'} ${path} failed (${res.status})`);
	}
	return res.status === 204 ? (undefined as T) : res.json();
}

const post = (path: string, body?: unknown) =>
	req(path, { method: 'POST', body: body ? JSON.stringify(body) : undefined });
const patch = (path: string, body?: unknown) =>
	req(path, { method: 'PATCH', body: body ? JSON.stringify(body) : undefined });
const del = (path: string) => req(path, { method: 'DELETE' });

export const api = {
	// ---------- Session ----------
	async sendMagicLink(email: string, invite?: string) {
		await post('/api/auth/magic-link', { email, invite });
		return true;
	},
	async logout() {
		await post('/api/auth/logout');
	},
	async passkeyAuthStart() {
		return req<{ challengeId: string; publicKey: RequestOptionsJSON }>('/api/auth/passkey/start', {
			method: 'POST'
		});
	},
	async passkeyAuthFinish(challengeId: string, credential: unknown) {
		const result = await req<{ ok: true; redirect: string }>('/api/auth/passkey/finish', {
			method: 'POST',
			body: JSON.stringify({ challengeId, credential })
		});
		// The backend just set the session cookie, but the client-side session
		// cache doesn't know that yet — without this, navigating to
		// `result.redirect` re-runs the root layout guard with the stale
		// signed-out state and immediately bounces back to /signin.
		db.session = { checked: true, signedIn: true, onboarded: result.redirect !== '/onboarding' };
		return result;
	},
	async passkeyRegisterStart() {
		return req<{ challengeId: string; publicKey: CreationOptionsJSON }>('/api/passkeys', {
			method: 'POST'
		});
	},
	async passkeyRegisterFinish(challengeId: string, credential: unknown, label?: string) {
		await post('/api/passkeys/finish', { challengeId, credential, label });
	},
	async listPasskeys() {
		return req<Passkey[]>('/api/passkeys');
	},
	async removePasskey(id: string) {
		await del(`/api/passkeys/${id}`);
	},
	async loadSession() {
		const s = await req<{
			signedIn: boolean;
			onboarded: boolean;
			user: { id: string; displayName: string; email: string; phoneSet: boolean } | null;
		}>('/api/session');
		db.session = { checked: true, signedIn: s.signedIn, onboarded: s.onboarded };
		db.currentUser = s.user;
		return s;
	},
	async completeOnboarding() {
		await post('/api/onboarding/complete');
		db.session.onboarded = true;
	},
	async setMyPhone(phone: string) {
		await post('/api/me/phone', { phone });
		if (db.currentUser) db.currentUser.phoneSet = true;
	},
	async inviteEmail(email: string) {
		await post('/api/invites', { email });
	},
	// Mints a shareable invite-link token to embed in SMS/WhatsApp invites, so a
	// number-only invitee can allowlist their own email at sign-in.
	async createInviteLink() {
		const { token } = await req<{ token: string }>('/api/invite-links', { method: 'POST' });
		return token;
	},

	// ---------- App bootstrap ----------
	/** Loads everything the app screens read straight off `db` (see db.svelte.ts). */
	async bootstrapApp() {
		const [calStatuses, friendAccess, outgoingPending, incomingRequests, standingAccess, polls] =
			await Promise.all([
				req<typeof db.calStatuses>('/api/calendar'),
				req<typeof db.friendAccess>('/api/access/mine'),
				req<typeof db.outgoingPending>('/api/access/outgoing'),
				req<typeof db.incomingRequests>('/api/access/incoming'),
				req<typeof db.standingAccess>('/api/access/standing'),
				req<Poll[]>('/api/polls')
			]);
		db.calStatuses = calStatuses;
		db.friendAccess = friendAccess;
		db.outgoingPending = outgoingPending;
		db.incomingRequests = incomingRequests;
		db.standingAccess = standingAccess;
		db.polls = polls;
	},

	// ---------- Reference data ----------
	async getFriends() {
		return req<Friend[]>('/api/friends');
	},
	async matchContacts(entries: { name: string; phone: string }[]) {
		const contacts = await req<Contact[]>('/api/contacts/match', {
			method: 'POST',
			body: JSON.stringify({ entries })
		});
		// The server never stores raw phone numbers, so if we ever need this
		// friend's number again (e.g. to notify them about an access
		// request), it has to come from here — cache it on this device now,
		// while we still have it.
		for (const c of contacts) {
			if (c.matched && c.userId) rememberPhone(c.userId, c.phone);
		}
		return contacts;
	},
	async addFriend(userId: string) {
		await post('/api/friends', { userId });
	},
	async searchEmail(email: string) {
		return req<EmailSearchResult>('/api/friends/search-email', {
			method: 'POST',
			body: JSON.stringify({ email })
		});
	},

	// ---------- Personal calendar ----------
	async getCalendarDays() {
		db.calStatuses = await req('/api/calendar');
		return db.calStatuses;
	},
	async setCalendarDayStatus(date: string, status: Availability | null) {
		const prevNote = db.calStatuses[date]?.note ?? '';
		if (!status && !prevNote) delete db.calStatuses[date];
		else db.calStatuses[date] = { status, note: prevNote };
		await patch(`/api/calendar/${date}`, { status });
	},
	async setCalendarDayNote(date: string, note: string) {
		const prevStatus = db.calStatuses[date]?.status ?? null;
		if (!prevStatus && !note) delete db.calStatuses[date];
		else db.calStatuses[date] = { status: prevStatus, note };
		await patch(`/api/calendar/${date}`, { note });
	},
	async clearCalendarDay(date: string) {
		delete db.calStatuses[date];
		await del(`/api/calendar/${date}`);
	},

	// ---------- Calendar access requests ----------
	async getFriendAccess() {
		db.friendAccess = await req('/api/access/mine');
		return db.friendAccess;
	},
	async getOutgoingPending() {
		db.outgoingPending = await req('/api/access/outgoing');
		return db.outgoingPending;
	},
	async requestCalendarAccess(friendId: string, scope: AccessScope, start: string, end: string) {
		db.outgoingPending[friendId] = true;
		await post(`/api/friends/${friendId}/access-request`, {
			scope,
			start: scope === 'range' ? start : null,
			end: scope === 'range' ? end : null
		});
	},
	async cancelOutgoingRequest(friendId: string) {
		delete db.outgoingPending[friendId];
		await del(`/api/access/outgoing/${friendId}`);
	},
	async getIncomingRequests() {
		db.incomingRequests = await req('/api/access/incoming');
		return db.incomingRequests;
	},
	async approveAccessRequest(requestId: string, detailLevel: DetailLevel) {
		await post(`/api/access/incoming/${requestId}/approve`, { detailLevel });
		db.incomingRequests = db.incomingRequests.filter((r) => r.id !== requestId);
		db.standingAccess = await req('/api/access/standing');
	},
	async denyAccessRequest(requestId: string) {
		await post(`/api/access/incoming/${requestId}/deny`);
		db.incomingRequests = db.incomingRequests.filter((r) => r.id !== requestId);
	},
	async getStandingAccess() {
		db.standingAccess = await req('/api/access/standing');
		return db.standingAccess;
	},
	async revokeStandingAccess(grantId: string) {
		db.standingAccess = db.standingAccess.filter((r) => r.id !== grantId);
		await del(`/api/access/standing/${grantId}`);
	},
	async getFriendCalendar(friendId: string) {
		return req<Record<string, Availability>>(`/api/friends/${friendId}/calendar`);
	},

	// ---------- Polls ----------
	async getPolls() {
		db.polls = await req('/api/polls');
		return db.polls;
	},
	async getPoll(id: string) {
		const poll = await req<Poll>(`/api/polls/${id}`);
		db.polls = [poll, ...db.polls.filter((p) => p.id !== id)];
		return poll;
	},
	async createPoll(input: {
		title: string;
		note: string;
		start: string;
		end: string;
		friendIds: string[];
		phoneInvitees: { phone: string; name: string }[];
	}) {
		const poll = await req<Poll>('/api/polls', { method: 'POST', body: JSON.stringify(input) });
		db.polls = [poll, ...db.polls];
		return poll;
	},
	async submitPollResponse(pollId: string, responses: Record<string, Availability>) {
		await post(`/api/polls/${pollId}/respond`, { responses });
		const poll = await req<Poll>(`/api/polls/${pollId}`);
		db.polls = db.polls.map((p) => (p.id === pollId ? poll : p));
	},
	async finalizePoll(pollId: string, start: string, end: string) {
		const poll = await post(`/api/polls/${pollId}/finalize`, { start, end });
		db.polls = db.polls.map((p) => (p.id === pollId ? (poll as Poll) : p));
	},
	async reopenPoll(pollId: string) {
		const poll = await post(`/api/polls/${pollId}/reopen`);
		db.polls = db.polls.map((p) => (p.id === pollId ? (poll as Poll) : p));
	},

	// ---------- Public (non-user) poll responses via token link ----------
	async getPublicPoll(token: string) {
		return req<Poll>(`/api/public/polls/${token}`);
	},
	async submitPublicPollResponse(token: string, responses: Record<string, Availability>) {
		await post(`/api/public/polls/${token}/respond`, { responses });
	}
};
