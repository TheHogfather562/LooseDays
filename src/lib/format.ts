export function pad(n: number): string {
	return String(n).padStart(2, '0');
}

export function dateStr(y: number, m: number, d: number): string {
	return `${y}-${pad(m + 1)}-${pad(d)}`;
}

export function dateRangeArr(start: string, end: string): string[] {
	const out: string[] = [];
	let cur = new Date(start + 'T00:00:00Z');
	const endD = new Date(end + 'T00:00:00Z');
	while (cur <= endD) {
		out.push(cur.toISOString().slice(0, 10));
		cur = new Date(cur.getTime() + 86400000);
	}
	return out;
}

export function fmtShort(d: string): string {
	return new Date(d + 'T00:00:00Z').toLocaleDateString('en-US', {
		month: 'short',
		day: 'numeric',
		timeZone: 'UTC'
	});
}

export function fmtWeekdayShort(d: string): string {
	return new Date(d + 'T00:00:00Z').toLocaleDateString('en-US', {
		weekday: 'short',
		month: 'short',
		day: 'numeric',
		timeZone: 'UTC'
	});
}

export function fmtRangeLabel(start: string, end: string): string {
	const y = new Date(end + 'T00:00:00Z').getFullYear();
	return `${fmtShort(start)} – ${fmtShort(end)}, ${y}`;
}

export function todayStr(): string {
	const d = new Date();
	return dateStr(d.getFullYear(), d.getMonth(), d.getDate());
}

export const WEEKDAY_LABELS = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
