// Mirrors the Postgres data model from the app spec. Mock/local implementations
// in this package should match these shapes so swapping in real API calls later
// is a small change, not a rewrite.

export type Availability = 'free' | 'busy' | 'maybe';

export interface User {
	id: string;
	displayName: string;
	email?: string;
	/** The server only ever stores a hash of the phone number, so this is a
	 * presence signal ("have I set one?"), never the number itself. */
	phoneSet?: boolean;
}

export interface Friend {
	id: string;
	displayName: string;
}

export interface Contact {
	id: string;
	name: string;
	phone: string;
	matched: boolean;
	userId: string | null;
	alreadyFriend?: boolean;
}

export interface EmailSearchResult {
	found: boolean;
	userId: string | null;
	displayName: string | null;
	alreadyFriend: boolean;
}

export type AccessScope = 'range' | 'standing';
export type DetailLevel = 'full' | 'overlap_only';

/** What a friend has granted the current user, keyed by friendId. */
export interface FriendAccessGrant {
	level: 'full' | 'overlap';
	scope: AccessScope;
	rangeStart?: string;
	rangeEnd?: string;
}

export interface CalendarDayEntry {
	status: Availability | null;
	note: string;
}

export type CalendarDays = Record<string, CalendarDayEntry>;

export interface IncomingRequest {
	id: string;
	requesterId: string;
	requesterName: string;
	scope: AccessScope;
	rangeStart: string | null;
	rangeEnd: string | null;
}

export interface StandingAccessGrant {
	id: string;
	friendId: string;
	friendName: string;
	detailLevel: DetailLevel;
}

export type InviteeStatus = 'invited' | 'responded';

export interface PollInvitee {
	id: string;
	userId: string | null;
	phone?: string;
	name: string;
	status: InviteeStatus;
	accessToken?: string;
	isMe?: boolean;
}

export type PollResponses = Record<string, Record<string, Availability>>;

export interface Poll {
	id: string;
	title: string;
	note: string;
	creatorId: string;
	rangeStart: string;
	rangeEnd: string;
	invitees: PollInvitee[];
	responses: PollResponses;
}

export interface Passkey {
	id: string;
	label: string;
	createdAt: string;
	lastUsedAt: string | null;
}
