<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { db } from '$lib/db.svelte';
	import { api } from '$lib/api';
	import { fmtShort } from '$lib/format';
	import type { Friend } from '$lib/types';

	let friends = $state<Friend[]>([]);
	$effect(() => {
		api.getFriends().then((f) => (friends = f));
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
</script>

<h1 class="m-0 px-[22px] pt-[26px] pb-1 font-display text-2xl font-semibold text-ink">Friends</h1>

<div class="flex gap-2.5 px-[18px] pt-3.5 pb-1">
	<a
		href={resolve('/friends/incoming')}
		class="flex-1 rounded-2xl border bg-white px-3.5 py-3 text-left no-underline"
		style="border-color:var(--color-line)"
	>
		<div class="text-[12.5px] font-semibold text-ink">Requests</div>
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
</div>

<div class="flex flex-col gap-2.5 px-[18px] pt-[18px]">
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
		</div>
	{/each}
</div>
