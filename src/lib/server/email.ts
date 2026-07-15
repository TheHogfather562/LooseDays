// Pluggable email sender. Set RESEND_API_KEY to send real mail via Resend
// (https://resend.com); otherwise the link is logged to the server console so
// you can test the whole flow without signing up for an email provider first.

import { env } from '$env/dynamic/private';

export async function sendMagicLinkEmail(to: string, url: string) {
	const from = env.EMAIL_FROM || 'Loose Days <onboarding@resend.dev>';
	const subject = 'Your Loose Days sign-in link';
	const text = `Tap to sign in to Loose Days:\n\n${url}\n\nThis link expires in 15 minutes and can only be used once.`;

	if (!env.RESEND_API_KEY) {
		console.log(`\n[loosedays] magic link for ${to}:\n  ${url}\n`);
		return;
	}

	const res = await fetch('https://api.resend.com/emails', {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${env.RESEND_API_KEY}`,
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ from, to, subject, text })
	});

	if (!res.ok) {
		console.error(`[loosedays] failed to send magic link email (${res.status}):`, await res.text());
		console.log(`[loosedays] magic link for ${to}:\n  ${url}\n`);
	}
}
