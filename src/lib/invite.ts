// The single source of truth for the "come join me on Loose Days" message
// handed off to SMS/WhatsApp. Kept here so onboarding, the add-friend screen,
// and anywhere else that surfaces an invite all say the same thing.

export function inviteMessage(): string {
	const origin = typeof window !== 'undefined' ? window.location.origin : '';
	return `Join me on Loose Days — a calendar for finding time with friends: ${origin}/signin`;
}
