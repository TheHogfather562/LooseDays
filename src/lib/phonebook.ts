// The server only ever stores a hash of a phone number (never the raw
// digits, per the app spec's privacy rule), so it can't hand a friend's
// number back to us later. Instead, whenever contact-matching surfaces a
// friend's number, we cache it here so screens that need to build an
// sms:/wa.me deep link later (e.g. "request calendar access") still have it
// on this device. If the cache is empty (different device, cleared storage),
// those screens just fall back to no phone-based deep link.

const STORAGE_KEY = 'loosedays:phonebook';

function readAll(): Record<string, string> {
	if (typeof localStorage === 'undefined') return {};
	try {
		return JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}');
	} catch {
		return {};
	}
}

export function rememberPhone(userId: string, phone: string): void {
	if (typeof localStorage === 'undefined') return;
	const all = readAll();
	all[userId] = phone;
	localStorage.setItem(STORAGE_KEY, JSON.stringify(all));
}

export function getRememberedPhone(userId: string): string | undefined {
	return readAll()[userId];
}
