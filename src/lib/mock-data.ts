// Local stand-in for a backend. Shapes match `types.ts` / the Postgres schema in
// the spec, so this module is the only thing that needs to change when a real
// API exists.

import type {
	CalendarDays,
	Contact,
	Friend,
	FriendAccessGrant,
	IncomingRequest,
	Poll,
	StandingAccessGrant,
	User
} from './types';

export const currentUser: User = { id: 'u0', displayName: 'You' };

export const friends: Friend[] = [
	{ id: 'u1', displayName: 'Priya Shah', phone: '+1 415-555-0101' },
	{ id: 'u2', displayName: 'Marcus Webb', phone: '+1 415-555-0102' },
	{ id: 'u3', displayName: 'Elena Cruz', phone: '+1 415-555-0103' },
	{ id: 'u4', displayName: 'Sam Okafor', phone: '+1 415-555-0104' },
	{ id: 'u5', displayName: 'Jonah Reyes', phone: '+1 415-555-0105' }
];

export const contacts: Contact[] = [
	{ id: 'c1', name: 'Priya Shah', phone: '+1 415-555-0101', matched: true, userId: 'u1' },
	{ id: 'c2', name: 'Marcus Webb', phone: '+1 415-555-0102', matched: true, userId: 'u2' },
	{ id: 'c3', name: 'Elena Cruz', phone: '+1 415-555-0103', matched: true, userId: 'u3' },
	{ id: 'c4', name: 'Dana Ruiz', phone: '+1 415-555-0199', matched: false, userId: null },
	{ id: 'c5', name: 'Tom Bergen', phone: '+1 415-555-0188', matched: false, userId: null }
];

// Access the current user has already been granted into a friend's calendar.
export const friendAccessOverrides: Record<string, FriendAccessGrant> = {
	u1: { level: 'full', scope: 'standing' },
	u2: { level: 'overlap', scope: 'range', rangeStart: '2026-07-01', rangeEnd: '2026-07-31' }
};

export const friendCalendars: Record<string, Record<string, string>> = {
	u1: {
		'2026-07-03': 'free',
		'2026-07-04': 'free',
		'2026-07-05': 'busy',
		'2026-07-11': 'free',
		'2026-07-12': 'maybe',
		'2026-07-18': 'busy',
		'2026-07-19': 'free',
		'2026-07-25': 'free',
		'2026-07-26': 'maybe'
	},
	u2: {
		'2026-07-02': 'free',
		'2026-07-04': 'free',
		'2026-07-09': 'busy',
		'2026-07-10': 'free',
		'2026-07-15': 'free',
		'2026-07-16': 'maybe',
		'2026-07-21': 'busy',
		'2026-07-28': 'free'
	}
};

export const initialIncomingRequests: IncomingRequest[] = [
	{
		id: 'r1',
		requesterId: 'u3',
		requesterName: 'Elena Cruz',
		scope: 'standing',
		rangeStart: null,
		rangeEnd: null
	},
	{
		id: 'r2',
		requesterId: 'u4',
		requesterName: 'Sam Okafor',
		scope: 'range',
		rangeStart: '2026-08-01',
		rangeEnd: '2026-08-07'
	}
];

export const initialStandingAccess: StandingAccessGrant[] = [
	{ id: 's1', friendId: 'u5', friendName: 'Jonah Reyes', detailLevel: 'full' },
	{ id: 's2', friendId: 'u1', friendName: 'Priya Shah', detailLevel: 'overlap_only' }
];

export const initialCalStatuses: CalendarDays = {
	'2026-07-02': { status: 'maybe', note: '' },
	'2026-07-04': { status: 'busy', note: '' },
	'2026-07-09': { status: 'busy', note: 'Deadline at work' },
	'2026-07-10': { status: 'maybe', note: '' },
	'2026-07-15': { status: 'busy', note: 'Team offsite' },
	'2026-07-20': { status: 'maybe', note: '' },
	'2026-07-21': { status: 'maybe', note: '' },
	'2026-07-27': { status: 'busy', note: '' }
};

export const initialPolls: Poll[] = [
	{
		id: 'p1',
		title: 'Cabin weekend',
		note: 'Lake house — need to book by Friday',
		creatorId: 'u0',
		rangeStart: '2026-08-07',
		rangeEnd: '2026-08-10',
		invitees: [
			{ id: 'i1', userId: 'u0', name: 'You', status: 'responded' },
			{ id: 'i2', userId: 'u1', name: 'Priya Shah', status: 'responded' },
			{ id: 'i3', userId: 'u2', name: 'Marcus Webb', status: 'responded' },
			{ id: 'i4', userId: 'u3', name: 'Elena Cruz', status: 'responded' },
			{ id: 'i5', userId: null, phone: '+1 555-0142', name: 'Dev (no account)', status: 'invited' }
		],
		responses: {
			i1: {
				'2026-08-07': 'free',
				'2026-08-08': 'free',
				'2026-08-09': 'maybe',
				'2026-08-10': 'busy'
			},
			i2: {
				'2026-08-07': 'free',
				'2026-08-08': 'free',
				'2026-08-09': 'free',
				'2026-08-10': 'busy'
			},
			i3: {
				'2026-08-07': 'busy',
				'2026-08-08': 'free',
				'2026-08-09': 'free',
				'2026-08-10': 'maybe'
			},
			i4: {
				'2026-08-07': 'maybe',
				'2026-08-08': 'free',
				'2026-08-09': 'free',
				'2026-08-10': 'free'
			}
		}
	},
	{
		id: 'p2',
		title: 'Dinner this week',
		note: '',
		creatorId: 'u1',
		rangeStart: '2026-07-15',
		rangeEnd: '2026-07-18',
		invitees: [
			{ id: 'i6', userId: 'u0', name: 'You', status: 'invited' },
			{ id: 'i7', userId: 'u4', name: 'Sam Okafor', status: 'responded' },
			{ id: 'i8', userId: 'u5', name: 'Jonah Reyes', status: 'responded' }
		],
		responses: {
			i7: {
				'2026-07-15': 'busy',
				'2026-07-16': 'free',
				'2026-07-17': 'free',
				'2026-07-18': 'maybe'
			},
			i8: { '2026-07-15': 'free', '2026-07-16': 'free', '2026-07-17': 'busy', '2026-07-18': 'free' }
		}
	},
	{
		id: 'p3',
		title: 'Beach day',
		note: '',
		creatorId: 'u0',
		rangeStart: '2026-06-20',
		rangeEnd: '2026-06-22',
		invitees: [
			{ id: 'i9', userId: 'u0', name: 'You', status: 'responded' },
			{ id: 'i10', userId: 'u2', name: 'Marcus Webb', status: 'responded' },
			{ id: 'i11', userId: 'u3', name: 'Elena Cruz', status: 'responded' }
		],
		responses: {
			i9: { '2026-06-20': 'free', '2026-06-21': 'free', '2026-06-22': 'free' },
			i10: { '2026-06-20': 'free', '2026-06-21': 'busy', '2026-06-22': 'free' },
			i11: { '2026-06-20': 'maybe', '2026-06-21': 'free', '2026-06-22': 'free' }
		}
	}
];
