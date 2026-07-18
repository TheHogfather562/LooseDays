// The single source of truth for the "come join me on Loose Days" message
// handed off to SMS/WhatsApp. Kept here so onboarding, the add-friend screen,
// and anywhere else that surfaces an invite all say the same thing.

// An optional invite token is threaded into the sign-in link so a brand-new
// recipient (whose email was never directly allowlisted) can redeem it and
// onboard themselves — essential for number-only invites, where we never learn
// their email up front.
export function inviteMessage(token?: string): string {
	const origin = typeof window !== 'undefined' ? window.location.origin : '';
	const signin = token
		? `${origin}/signin?invite=${encodeURIComponent(token)}`
		: `${origin}/signin`;
	return `Join me on Loose Days — a calendar for finding time with friends: ${signin}`;
}
