<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import StatusDot from '$lib/components/StatusDot.svelte';
	import { db } from '$lib/db.svelte';
	import { dateRangeArr, fmtRangeLabel, fmtShort } from '$lib/format';
	import { computeBest, myInvitee } from '$lib/polls';

	const pollId = $derived(page.params.id!);
	const poll = $derived(db.polls.find((p) => p.id === pollId));
	const mine = $derived(poll ? myInvitee(poll) : undefined);

	const days = $derived(poll ? dateRangeArr(poll.rangeStart, poll.rangeEnd) : []);
	const best = $derived(poll ? computeBest(days, poll.invitees, poll.responses) : null);
	const bestRangeLabel = $derived(
		best
			? best.startIdx === best.endIdx
				? fmtShort(days[best.startIdx])
				: `${fmtShort(days[best.startIdx])} – ${fmtShort(days[best.endIdx])}`
			: ''
	);

	function initials(name: string) {
		return name
			.split(' ')
			.map((w) => w[0])
			.join('')
			.slice(0, 2)
			.toUpperCase();
	}

	function cellColor(status: string | undefined) {
		if (status === 'busy') return 'var(--color-busy)';
		if (status === 'maybe') return 'var(--color-maybe)';
		if (status === 'free') return 'var(--color-free)';
		return 'var(--color-pending)';
	}

	// A letter inside each cell so availability isn't conveyed by colour alone —
	// free/maybe/busy are otherwise only distinguishable by hue.
	function cellLetter(status: string | undefined) {
		if (status === 'busy') return 'B';
		if (status === 'maybe') return 'M';
		if (status === 'free') return 'F';
		return '';
	}
</script>

<BackHeader
	title={poll?.title ?? ''}
	href={resolve('/polls')}
	subtitle={poll ? fmtRangeLabel(poll.rangeStart, poll.rangeEnd) : ''}
/>

{#if poll && mine}
	<div class="flex justify-end px-[22px] pt-1">
		<a
			href={resolve('/(app)/polls/[id]/respond', { id: poll.id })}
			class="text-[12px] font-semibold text-accent"
		>
			{mine.status === 'responded' ? 'Edit my response' : 'Add my response'}
		</a>
	</div>
{/if}

{#if best}
	<div
		class="mx-[18px] mt-3.5 flex flex-col gap-0.5 rounded-xl px-3.5 py-3"
		style="background:var(--color-overlap-bg)"
	>
		<span class="text-[11.5px] font-semibold" style="color:var(--color-overlap-text)">
			Best overlap
		</span>
		<span class="text-[13.5px] text-ink-soft">{bestRangeLabel}</span>
	</div>
{/if}

{#if poll}
	<div class="mt-4 overflow-x-auto px-[18px] pb-[18px]">
		<div
			class="grid items-center gap-x-1 gap-y-1.5"
			style={`grid-template-columns:64px repeat(${poll.invitees.length}, 40px)`}
		>
			<div></div>
			{#each poll.invitees as inv (inv.id)}
				<div class="pb-2 text-center text-[11px] font-semibold text-subtext">
					{initials(inv.name)}
				</div>
			{/each}
			{#each days as d, di (d)}
				{@const inBest = !!best && di >= best.startIdx && di <= best.endIdx}
				<div
					class="rounded-md px-1 py-1.5 text-[11.5px] text-ink"
					style="font-weight:{inBest ? 700 : 400}; background:{inBest
						? 'var(--color-overlap-bg)'
						: 'transparent'}"
				>
					{fmtShort(d)}
				</div>
				{#each poll.invitees as inv (inv.id)}
					{@const st = poll.responses[inv.id]?.[d]}
					<div
						class="mx-auto box-border flex h-8 w-8 items-center justify-center rounded-lg text-[10px] font-bold"
						style="background:{cellColor(st)}; color:{st
							? 'rgba(255,255,255,0.9)'
							: 'var(--color-muted)'}; border:{inBest ? '2px solid var(--color-accent)' : 'none'}"
						title={st ?? 'pending'}
					>
						{cellLetter(st)}
					</div>
				{/each}
			{/each}
		</div>
	</div>
{/if}

<div class="flex gap-3.5 px-[22px] pb-5 text-[11px] text-subtext">
	<div class="flex items-center gap-1.5">
		<StatusDot status="free" size={12} /><span>F · free</span>
	</div>
	<div class="flex items-center gap-1.5">
		<StatusDot status="maybe" size={12} /><span>M · maybe</span>
	</div>
	<div class="flex items-center gap-1.5">
		<StatusDot status="busy" size={12} /><span>B · busy</span>
	</div>
	<div class="flex items-center gap-1.5">
		<span class="inline-block h-3 w-3 rounded-full" style="background:var(--color-pending)"></span>
		<span>pending</span>
	</div>
</div>
