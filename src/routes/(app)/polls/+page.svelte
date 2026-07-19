<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { db } from '$lib/db.svelte';
	import { fmtRangeLabel, fmtShort } from '$lib/format';
	import { myInvitee } from '$lib/polls';
	import type { Poll } from '$lib/types';

	function finalizedLabel(poll: Poll): string {
		if (!poll.finalizedStart || !poll.finalizedEnd) return '';
		return poll.finalizedStart === poll.finalizedEnd
			? fmtShort(poll.finalizedStart)
			: `${fmtShort(poll.finalizedStart)} – ${fmtShort(poll.finalizedEnd)}`;
	}

	function open(poll: Poll) {
		const mine = myInvitee(poll);
		if (mine && mine.status === 'invited')
			goto(resolve('/(app)/polls/[id]/respond', { id: poll.id }));
		else goto(resolve('/(app)/polls/[id]/results', { id: poll.id }));
	}
</script>

<div class="flex items-center justify-between px-[22px] pt-[26px] pb-1">
	<h1 class="m-0 font-display text-2xl font-semibold text-ink">Polls</h1>
	<a
		href={resolve('/polls/new')}
		class="rounded-[10px] border-none bg-accent px-3.5 py-2.5 text-[13px] font-semibold text-white no-underline"
	>
		+ New poll
	</a>
</div>
<p class="m-0 px-[22px] pt-1.5 pb-[18px] text-[12.5px] text-subtext-2">
	Find the range that works for everyone
</p>

<div class="flex flex-col gap-3 px-[18px]">
	{#if db.polls.length === 0}
		<div
			class="mt-2 flex flex-col items-center gap-3 rounded-2xl border border-dashed px-6 py-9 text-center"
			style="border-color:var(--color-line)"
		>
			<p class="m-0 text-[13px] font-semibold text-ink">No polls yet</p>
			<p class="m-0 text-[12px] leading-relaxed text-subtext-2">
				Start a poll to find a date that works for everyone — invite friends or anyone by phone
				number.
			</p>
			<a
				href={resolve('/polls/new')}
				class="rounded-[10px] border-none bg-accent px-4 py-2.5 text-[12.5px] font-semibold text-white no-underline"
			>
				+ Create your first poll
			</a>
		</div>
	{/if}
	{#each db.polls as poll (poll.id)}
		{@const mine = myInvitee(poll)}
		{@const pendingCount = poll.invitees.filter((i) => i.status === 'invited').length}
		{@const needsResponse = mine?.status === 'invited'}
		<button
			onclick={() => open(poll)}
			class="flex cursor-pointer flex-col gap-2 rounded-2xl border bg-white p-4 text-left"
			style="border-color:var(--color-line)"
		>
			<div class="flex items-baseline justify-between">
				<span class="font-display text-[15px] font-semibold text-ink">{poll.title}</span>
				<span
					class="rounded-full px-2.5 py-1 text-[11px] font-semibold"
					style={needsResponse
						? 'color:#fff;background:var(--color-accent)'
						: 'color:var(--color-ink);background:var(--color-chip)'}
				>
					{needsResponse ? 'Respond' : 'View results'}
				</span>
			</div>
			<span class="text-[12.5px] text-subtext">{fmtRangeLabel(poll.rangeStart, poll.rangeEnd)}</span
			>
			{#if poll.finalizedAt}
				<span class="text-[12px] font-semibold" style="color:var(--color-accent)">
					✓ Locked in · {finalizedLabel(poll)}
				</span>
			{/if}
			<span class="text-xs text-muted">
				{poll.invitees.length} invited · {poll.invitees.length - pendingCount} responded
			</span>
		</button>
	{/each}
</div>
