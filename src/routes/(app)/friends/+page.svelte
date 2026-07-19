<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { db } from '$lib/db.svelte';
	import { api } from '$lib/api';
	import { withToast } from '$lib/toast.svelte';
	import { fmtShort } from '$lib/format';
	import type { Friend } from '$lib/types';

	let friends = $state<Friend[]>([]);
	let loaded = $state(false);
	$effect(() => {
		api.getFriends().then((f) => {
			friends = f;
			loaded = true;
		});
	});

	const incomingCountLabel = $derived(
		db.incomingRequests.length ? `${db.incomingRequests.length} pending` : 'None pending'
	);
	const standingCountLabel = $derived(
		db.standingAccess.length ? `${db.standingAccess.length} people` : 'Nobody yet'
	);

	function rowFor(f: Friend) {
		const access = db.friendAccess[f.id];
		const pending = db.outgoingPending[f.id];
		const actionLabel = access ? 'View calendar' : pending ? 'Requested' : 'Request access';
		const levelLabel = access
			? access.level === 'full'
				? 'Full access'
				: 'Overlap-only access'
			: '';
		const subLabel = access
			? access.scope === 'range'
				? `${levelLabel} · through ${fmtShort(access.rangeEnd!)}`
				: `${levelLabel} · ongoing`
			: pending
				? 'Waiting on approval'
				: 'No access yet';
		const showRequestMore = !!access && access.scope === 'range' && !pending;
		return { access, pending, actionLabel, subLabel, showRequestMore };
	}

	function onAction(f: Friend) {
		const r = rowFor(f);
		if (r.access) goto(resolve('/(app)/friends/view/[friendId]', { friendId: f.id }));
		else if (!r.pending) goto(resolve('/(app)/friends/request/[friendId]', { friendId: f.id }));
	}

	async function cancelRequest(f: Friend) {
		const ok = await withToast(() => api.cancelOutgoingRequest(f.id), {
			success: 'Request withdrawn.',
			error: "Couldn't withdraw the request — try again."
		});
		// api.* clears the pending flag optimistically; restore it if it failed.
		if (!ok) await api.getOutgoingPending().catch(() => {});
	}
</script>

<div class="flex items-center justify-between px-[22px] pt-[26px] pb-1">
	<h1 class="m-0 font-display text-2xl font-semibold text-ink">Friends</h1>
	<a
		href={resolve('/friends/add')}
		class="rounded-lg border-none bg-accent px-3 py-1.5 text-xs font-semibold text-white no-underline"
	>
		+ Add friend
	</a>
</div>

<div class="flex gap-2.5 px-[18px] pt-3.5 pb-1">
	<a
		href={resolve('/friends/incoming')}
		class="relative flex-1 rounded-2xl border px-3.5 py-3 text-left no-underline"
		style={db.incomingRequests.length
			? 'border-color:var(--color-accent); background:var(--color-overlap-bg)'
			: 'border-color:var(--color-line); background:#fff'}
	>
		<div class="flex items-center gap-1.5">
			<span class="text-[12.5px] font-semibold text-ink">Requests</span>
			{#if db.incomingRequests.length}
				<span
					class="flex h-[15px] min-w-[15px] items-center justify-center rounded-full px-1 text-[9px] font-bold text-white"
					style="background:var(--color-accent)"
				>
					{db.incomingRequests.length > 9 ? '9+' : db.incomingRequests.length}
				</span>
			{/if}
		</div>
		<div class="mt-0.5 text-[11px] text-subtext-2">{incomingCountLabel}</div>
	</a>
	<a
		href={resolve('/friends/who-can-see')}
		class="flex-1 rounded-2xl border bg-white px-3.5 py-3 text-left no-underline"
		style="border-color:var(--color-line)"
	>
		<div class="text-[12.5px] font-semibold text-ink">Who can see me</div>
		<div class="mt-0.5 text-[11px] text-subtext-2">{standingCountLabel}</div>
	</a>
	<a
		href={resolve('/account')}
		class="flex-1 rounded-2xl border bg-white px-3.5 py-3 text-left no-underline"
		style="border-color:var(--color-line)"
	>
		<div class="text-[12.5px] font-semibold text-ink">Account</div>
		<div class="mt-0.5 text-[11px] text-subtext-2">Passkeys, sign out</div>
	</a>
</div>

<div class="flex flex-col gap-2.5 px-[18px] pt-[18px]">
	{#if loaded && friends.length === 0}
		<div
			class="mt-2 flex flex-col items-center gap-3 rounded-2xl border border-dashed px-6 py-9 text-center"
			style="border-color:var(--color-line)"
		>
			<p class="m-0 text-[13px] font-semibold text-ink">No friends yet</p>
			<p class="m-0 text-[12px] leading-relaxed text-subtext-2">
				Add friends by phone number or email to start sharing calendars.
			</p>
			<a
				href={resolve('/friends/add')}
				class="rounded-[10px] border-none bg-accent px-4 py-2.5 text-[12.5px] font-semibold text-white no-underline"
			>
				+ Add your first friend
			</a>
		</div>
	{/if}
	{#each friends as f (f.id)}
		{@const r = rowFor(f)}
		<div
			class="flex flex-col gap-2 rounded-[14px] border bg-white px-3.5 py-3"
			style="border-color:var(--color-line)"
		>
			<div class="flex items-center justify-between">
				<div>
					<div class="text-[13.5px] font-semibold text-ink">{f.displayName}</div>
					<div class="text-[11.5px] text-muted">{r.subLabel}</div>
				</div>
				<button
					onclick={() => onAction(f)}
					disabled={r.pending}
					class="rounded-lg px-3 py-2 text-xs font-semibold"
					style={r.access
						? 'border:none; background:var(--color-chip); color:var(--color-accent); cursor:pointer'
						: r.pending
							? 'border:1px solid var(--color-line); background:#fff; color:var(--color-muted); cursor:default'
							: 'border:1px solid var(--color-accent); background:#fff; color:var(--color-accent); cursor:pointer'}
				>
					{r.actionLabel}
				</button>
			</div>
			{#if r.showRequestMore}
				<a
					href={resolve('/(app)/friends/request/[friendId]', { friendId: f.id })}
					class="self-start text-[11.5px] font-semibold text-accent"
				>
					Request another range
				</a>
			{/if}
			{#if r.pending}
				<button
					onclick={() => cancelRequest(f)}
					class="cursor-pointer self-start border-none bg-transparent p-0 text-[11.5px] font-semibold"
					style="color:var(--color-danger)"
				>
					Cancel request
				</button>
			{/if}
		</div>
	{/each}
</div>
