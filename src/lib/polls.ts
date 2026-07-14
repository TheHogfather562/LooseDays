import type { Poll, PollInvitee } from './types';

export interface BestOverlap {
	counts: number[];
	startIdx: number;
	endIdx: number;
}

export function computeBest(
	days: string[],
	invitees: PollInvitee[],
	responses: Poll['responses']
): BestOverlap | null {
	const counts = days.map(
		(d) => invitees.filter((inv) => responses[inv.id]?.[d] === 'free').length
	);
	const max = Math.max(0, ...counts);
	if (max === 0) return null;

	let bestStart = -1;
	let bestLen = 0;
	let curStart = -1;
	let curLen = 0;
	for (let i = 0; i <= counts.length; i++) {
		if (i < counts.length && counts[i] === max) {
			if (curLen === 0) curStart = i;
			curLen++;
		} else {
			if (curLen > bestLen) {
				bestLen = curLen;
				bestStart = curStart;
			}
			curLen = 0;
		}
	}
	return { counts, startIdx: bestStart, endIdx: bestStart + bestLen - 1 };
}

export function myInvitee(poll: Poll): PollInvitee | undefined {
	return poll.invitees.find((i) => i.userId === 'u0');
}
