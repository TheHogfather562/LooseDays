<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import { api } from '$lib/api';
	import { withToast } from '$lib/toast.svelte';
	import { smsLink, whatsappLink } from '$lib/links';
	import { getRememberedPhone } from '$lib/phonebook';
	import { todayStr } from '$lib/format';
	import type { AccessScope, Friend } from '$lib/types';

	let friends = $state<Friend[]>([]);
	$effect(() => {
		api.getFriends().then((f) => (friends = f));
	});

	const friendId = $derived(page.params.friendId);
	const friend = $derived(friends.find((f) => f.id === friendId));
	// The server never stores a friend's raw phone number — this only works
	// if this device matched their contact locally at some point (onboarding
	// or the friends contact-match flow), which caches it here.
	const friendPhone = $derived(friendId ? getRememberedPhone(friendId) : undefined);

	let scope = $state<AccessScope>('range');
	let start = $state('');
	let end = $state('');
	let sent = $state(false);
	let sending = $state(false);

	const today = todayStr();
	const rangeBackwards = $derived(scope === 'range' && !!start && !!end && end < start);
	const sendDisabled = $derived(
		sending || (scope === 'range' ? !(start && end) || rangeBackwards : false)
	);

	async function send() {
		if (sendDisabled || !friendId) return;
		sending = true;
		const ok = await withToast(() => api.requestCalendarAccess(friendId, scope, start, end), {
			error: "Couldn't send the request — try again."
		});
		sending = false;
		if (ok) sent = true;
		// api.* flips db.outgoingPending optimistically; undo it if the send failed.
		else await api.getOutgoingPending().catch(() => {});
	}

	const reqMsg = `Can you approve my request to see your Loose Days calendar? ${
		typeof window !== 'undefined' ? window.location.origin : ''
	}/friends/incoming`;
</script>

<BackHeader title="Request access" href={resolve('/friends')} />

{#if sent}
	<div class="px-[22px]">
		<div
			class="mt-1.5 flex items-center justify-between rounded-[14px] border bg-white p-3.5"
			style="border-color:var(--color-line)"
		>
			<div>
				<div class="text-[13.5px] font-semibold text-ink">{friend?.displayName ?? ''}</div>
				<div class="text-[11.5px] text-muted">Notify them to approve</div>
			</div>
			{#if friendPhone}
				<div class="flex gap-2">
					<a
						href={smsLink(friendPhone, reqMsg)}
						rel="external"
						class="rounded-lg border px-2.5 py-[7px] text-xs font-semibold text-ink no-underline"
						style="border-color:var(--color-line)"
					>
						SMS
					</a>
					<a
						href={whatsappLink(friendPhone, reqMsg)}
						rel="external"
						class="rounded-lg border px-2.5 py-[7px] text-xs font-semibold text-ink no-underline"
						style="border-color:var(--color-line)"
					>
						WhatsApp
					</a>
				</div>
			{:else}
				<div class="text-[11.5px] text-muted">
					No cached number for them on this device — ask them to check the app directly.
				</div>
			{/if}
		</div>
		<a
			href={resolve('/friends')}
			class="mt-4 block w-full rounded-[10px] border-none bg-accent py-3 text-center text-sm font-semibold text-white no-underline"
		>
			Done
		</a>
	</div>
{:else}
	<div class="flex flex-col gap-4 px-[22px] pt-2">
		<div>
			<span class="text-xs font-semibold text-subtext">Friend</span>
			<div class="mt-1.5 text-[14.5px] font-semibold text-ink">{friend?.displayName ?? ''}</div>
		</div>
		<div class="flex flex-col gap-2">
			<span class="text-xs font-semibold text-subtext">Scope</span>
			<div class="flex gap-2">
				<button
					onclick={() => (scope = 'range')}
					class="flex-1 cursor-pointer rounded-[10px] py-2.5 text-[12.5px] font-semibold"
					style={scope === 'range'
						? 'border:none;background:var(--color-accent);color:#fff'
						: 'border:1px solid var(--color-line);background:#fff;color:var(--color-ink)'}
				>
					Specific range
				</button>
				<button
					onclick={() => (scope = 'standing')}
					class="flex-1 cursor-pointer rounded-[10px] py-2.5 text-[12.5px] font-semibold"
					style={scope === 'standing'
						? 'border:none;background:var(--color-accent);color:#fff'
						: 'border:1px solid var(--color-line);background:#fff;color:var(--color-ink)'}
				>
					Ongoing
				</button>
			</div>
		</div>
		{#if scope === 'range'}
			<div class="flex gap-3">
				<label class="flex flex-1 flex-col gap-1.5">
					<span class="text-xs font-semibold text-subtext">Start</span>
					<input
						type="date"
						bind:value={start}
						min={today}
						class="rounded-[10px] border px-2.5 py-2.5 text-[13px] text-ink"
						style="border-color:var(--color-line)"
					/>
				</label>
				<label class="flex flex-1 flex-col gap-1.5">
					<span class="text-xs font-semibold text-subtext">End</span>
					<input
						type="date"
						bind:value={end}
						min={start || today}
						class="rounded-[10px] border px-2.5 py-2.5 text-[13px] text-ink"
						style="border-color:var(--color-line)"
					/>
				</label>
			</div>
			{#if rangeBackwards}
				<p class="m-0 text-[11.5px] leading-relaxed" style="color:var(--color-danger)">
					The end date can't be before the start date.
				</p>
			{/if}
		{/if}
		<button
			onclick={send}
			disabled={sendDisabled}
			class="w-full rounded-[10px] border-none py-3 text-sm font-semibold text-white"
			style="background:{sendDisabled
				? 'var(--color-faint)'
				: 'var(--color-accent)'}; cursor:{sendDisabled ? 'default' : 'pointer'}"
		>
			{sending ? 'Sending…' : 'Send request'}
		</button>
	</div>
{/if}
