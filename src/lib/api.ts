// Thin async wrapper around the local store, shaped like the API calls this
// app will eventually make against the real backend (see spec.md). Every
// function here is a candidate to become a `fetch()` call later — components
// should always go through this module rather than importing mock-data or
// db.svelte directly.

import { db, persist } from './db.svelte';
import { contacts, currentUser, friendCalendars, friends } from './mock-data';
import type { Availability, AccessScope, DetailLevel, Poll, PollInvitee } from './types';

function delay<T>(value: T): Promise<T> {
	return Promise.resolve(value);
}

export const api = {
	// ---------- Session ----------
	async sendMagicLink(_email: string) {
		return delay(true);
	},
	async completeSignIn() {
		db.session.signedIn = true;
		persist();
	},
	async completeOnboarding() {
		db.session.onboarded = true;
		persist();
	},

	// ---------- Reference data ----------
	async getCurrentUser() {
		return delay(currentUser);
	},
	async getFriends() {
		return delay(friends);
	},
	async getContacts() {
		return delay(contacts);
	},
	async toggleContactAdded(contactId: string) {
		db.onboardingAdded[contactId] = true;
		persist();
	},

	// ---------- Personal calendar ----------
	async getCalendarDays() {
		return delay(db.calStatuses);
	},
	async setCalendarDayStatus(date: string, status: Availability | null) {
		const prevNote = db.calStatuses[date]?.note ?? '';
		if (!status && !prevNote) delete db.calStatuses[date];
		else db.calStatuses[date] = { status, note: prevNote };
		persist();
	},
	async setCalendarDayNote(date: string, note: string) {
		const prevStatus = db.calStatuses[date]?.status ?? null;
		if (!prevStatus && !note) delete db.calStatuses[date];
		else db.calStatuses[date] = { status: prevStatus, note };
		persist();
	},
	async clearCalendarDay(date: string) {
		delete db.calStatuses[date];
		persist();
	},

	// ---------- Calendar access requests ----------
	async getFriendAccess() {
		return delay(db.friendAccess);
	},
	async getOutgoingPending() {
		return delay(db.outgoingPending);
	},
	async requestCalendarAccess(friendId: string, _scope: AccessScope, _start: string, _end: string) {
		db.outgoingPending[friendId] = true;
		persist();
	},
	async getIncomingRequests() {
		return delay(db.incomingRequests);
	},
	async approveAccessRequest(requestId: string, detailLevel: DetailLevel) {
		const req = db.incomingRequests.find((r) => r.id === requestId);
		if (!req) return;
		db.standingAccess.push({
			id: 's' + Date.now(),
			friendId: req.requesterId,
			friendName: req.requesterName,
			detailLevel
		});
		db.incomingRequests = db.incomingRequests.filter((r) => r.id !== requestId);
		persist();
	},
	async denyAccessRequest(requestId: string) {
		db.incomingRequests = db.incomingRequests.filter((r) => r.id !== requestId);
		persist();
	},
	async getStandingAccess() {
		return delay(db.standingAccess);
	},
	async revokeStandingAccess(grantId: string) {
		db.standingAccess = db.standingAccess.filter((r) => r.id !== grantId);
		persist();
	},
	async getFriendCalendar(friendId: string) {
		return delay(friendCalendars[friendId] ?? {});
	},

	// ---------- Polls ----------
	async getPolls() {
		return delay(db.polls);
	},
	async getPoll(id: string) {
		return delay(db.polls.find((p) => p.id === id) ?? null);
	},
	async createPoll(input: {
		title: string;
		note: string;
		start: string;
		end: string;
		friendIds: string[];
		phoneChips: string[];
	}) {
		const selectedFriends = friends.filter((f) => input.friendIds.includes(f.id));
		const invitees: PollInvitee[] = [
			{ id: 'i_me', userId: 'u0', name: 'You', status: 'responded' },
			...selectedFriends.map((f) => ({
				id: 'i_' + f.id,
				userId: f.id,
				name: f.displayName,
				status: 'invited' as const
			})),
			...input.phoneChips.map((ph, idx) => ({
				id: 'i_ph' + idx,
				userId: null,
				phone: ph,
				name: ph,
				status: 'invited' as const
			}))
		];
		const poll: Poll = {
			id: 'p' + Date.now(),
			title: input.title || 'Untitled poll',
			note: input.note,
			creatorId: 'u0',
			rangeStart: input.start,
			rangeEnd: input.end,
			invitees,
			responses: {}
		};
		db.polls = [poll, ...db.polls];
		persist();
		return poll;
	},
	async submitPollResponse(pollId: string, responses: Record<string, Availability>) {
		const poll = db.polls.find((p) => p.id === pollId);
		if (!poll) return;
		const mine = poll.invitees.find((i) => i.userId === 'u0');
		if (!mine) return;
		poll.responses[mine.id] = responses;
		mine.status = 'responded';
		persist();
	}
};
