// Deep links that hand off to the user's own messaging app — Loose Days never
// sends SMS/WhatsApp itself. Callers should only render these when they
// actually have a number (see InviteLinks.svelte); an empty `phone` here would
// otherwise produce a recipient-less `sms:` link that silently goes nowhere.

export function smsLink(phone: string, body: string): string {
	// Keep digits and a leading +, drop spaces/dashes/parens that some OS SMS
	// handlers choke on. `?body=` (RFC 5724) is the widely-supported form.
	const num = phone.replace(/[^\d+]/g, '');
	return `sms:${num}?body=${encodeURIComponent(body)}`;
}

export function whatsappLink(phone: string, body: string): string {
	// wa.me wants bare digits, no leading +.
	return `https://wa.me/${phone.replace(/\D/g, '')}?text=${encodeURIComponent(body)}`;
}
