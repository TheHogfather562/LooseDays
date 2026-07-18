<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import AppShell from '$lib/components/AppShell.svelte';
	import { api } from '$lib/api';
	import { passkeysSupported, signInWithPasskey } from '$lib/webauthn';

	let email = $state('');
	let sent = $state(false);
	let passkeyError = $state('');
	let passkeyPending = $state(false);

	const disabled = $derived(!email.includes('@'));

	async function send() {
		if (disabled) return;
		await api.sendMagicLink(email);
		sent = true;
	}

	async function resend() {
		await api.sendMagicLink(email);
	}

	async function withPasskey() {
		passkeyError = '';
		passkeyPending = true;
		try {
			const { redirect } = await signInWithPasskey();
			await goto(redirect === '/onboarding' ? resolve('/onboarding') : resolve('/calendar'));
		} catch (err) {
			console.error('passkey sign-in failed', err);
			const detail = err instanceof Error ? err.message : '';
			passkeyError = detail
				? `Couldn't sign in with that passkey — ${detail}`
				: "Couldn't sign in with that passkey — try again or use your email link.";
		} finally {
			passkeyPending = false;
		}
	}
</script>

<AppShell>
	<div class="flex min-h-[520px] flex-col justify-center px-[30px] py-8">
		<div class="mb-9 flex items-center gap-2">
			<span class="h-2.5 w-2.5 rounded-full bg-accent"></span>
			<span class="font-display text-[15px] font-semibold tracking-[0.01em] text-ink"
				>Loose Days</span
			>
		</div>

		{#if sent}
			<div
				class="relative mb-[18px] h-8 w-11 rounded-md"
				style="border:2px solid var(--color-faint-2)"
			>
				<div
					class="absolute top-0 right-0 left-0 h-0.5"
					style="background:var(--color-faint-2)"
				></div>
			</div>
			<h1 class="m-0 mb-2 font-display text-[21px] font-semibold text-ink">Check your email</h1>
			<p class="m-0 mb-[22px] text-[13px] leading-relaxed text-subtext">
				We sent a sign-in link to {email}. Open it on this device to continue.
			</p>
			<button
				onclick={resend}
				class="cursor-pointer self-start border-none bg-transparent p-0 text-[13px] font-semibold text-accent"
			>
				Resend link
			</button>
		{:else}
			<h1 class="m-0 mb-2 font-display text-[21px] font-semibold text-ink">Sign in</h1>
			<p class="m-0 mb-[22px] text-[13px] leading-relaxed text-subtext">
				Enter your email — we'll send a magic link, no password needed.
			</p>
			<label class="mb-4 flex flex-col gap-1.5">
				<span class="text-[12px] font-semibold text-subtext">Email</span>
				<input
					type="email"
					placeholder="you@example.com"
					bind:value={email}
					class="rounded-[10px] border px-3.5 py-3 text-sm text-ink"
					style="border-color:var(--color-line)"
				/>
			</label>
			<button
				onclick={send}
				{disabled}
				class="w-full rounded-[10px] border-none py-3 text-sm font-semibold text-white"
				style="background:{disabled
					? 'var(--color-faint)'
					: 'var(--color-accent)'}; cursor:{disabled ? 'default' : 'pointer'}"
			>
				Send magic link
			</button>

			{#if passkeysSupported()}
				<button
					onclick={withPasskey}
					disabled={passkeyPending}
					class="mt-3 w-full rounded-[10px] border py-3 text-sm font-semibold text-ink"
					style="border-color:var(--color-line); cursor:{passkeyPending ? 'default' : 'pointer'}"
				>
					{passkeyPending ? 'Waiting for passkey…' : 'Sign in with a passkey'}
				</button>
				{#if passkeyError}
					<p class="mt-2 mb-0 text-[11.5px] leading-relaxed" style="color:var(--color-danger)">
						{passkeyError}
					</p>
				{/if}
			{/if}

			<p class="mt-4 mb-0 text-[11.5px] leading-relaxed text-muted-2">
				Sign-up is invite-only right now — ask a friend already on Loose Days to add you.
			</p>
		{/if}
	</div>
</AppShell>
