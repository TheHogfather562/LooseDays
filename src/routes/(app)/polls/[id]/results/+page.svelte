<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import StatusDot from '$lib/components/StatusDot.svelte';
	import { db } from '$lib/db.svelte';
	import { api } from '$lib/api';
	import { withToast } from '$lib/toast.svelte';
	import { dateRangeArr, fmtRangeLabel, fmtShort } from '$lib/format';
	import { computeBest, myInvitee } from '$lib/polls';

	const pollId = $derived(page.params.id!);
	const poll = $derived(db.polls.find((p) => p.id === pollId));
	const mine = $derived(poll ? myInvitee(poll) : undefined);
	const isCreator = $derived(!!poll && poll.creatorId === db.currentUser?.id);

	const days = $derived(poll ? dateRangeArr(poll.rangeStart, poll.rangeEnd) : []);
	const best = $derived(poll ? computeBest(days, poll.invitees, poll.responses) : null);
	const bestRangeLabel = $derived(
		best
			? best.startIdx === best.endIdx
				? fmtShort(days[best.startIdx])
				: `${fmtShort(days[best.startIdx])} – ${fmtShort(days[best.endIdx])}`
			: ''
	);

	const finalized = $derived(!!poll?.finalizedAt);
	const finalizedLabel = $derived(
		poll?.finalizedStart && poll?.finalizedEnd
			? poll.finalizedStart === poll.finalizedEnd
				? fmtShort(poll.finalizedStart)
				: `${fmtShort(poll.finalizedStart)} – ${fmtShort(poll.finalizedEnd)}`
			: ''
	);

	// The creator's proposed lock-in range, prefilled once from the best overlap
	// (or the poll's own range if nobody's free anywhere yet), then adjustable.
	let finStart = $state('');
	let finEnd = $state('');
	let seededFin = $state(false);
	let busy = $state(false);
	$effect(() => {
		if (seededFin || !poll) return;
		if (best) {
			finStart = days[best.startIdx];
			finEnd = days[best.endIdx];
		} else {
			finStart = poll.rangeStart;
			finEnd = poll.rangeEnd;
		}
		seededFin = true;
	});
	const finBackwards = $derived(!!finStart && !!finEnd && finEnd < finStart);

	async function finalize() {
		if (!poll || busy || !finStart || !finEnd || finBackwards) return;
		const id = poll.id;
		busy = true;
		await withToast(() => api.finalizePoll(id, finStart, finEnd), {
			success: 'Dates locked in.',
			error: "Couldn't finalize — try again."
		});
		busy = false;
	}

	async function reopen() {
		if (!poll || busy) return;
		const id = poll.id;
		busy = true;
		await withToast(() => api.reopenPoll(id), {
			success: 'Poll reopened.',
			error: "Couldn't reopen — try again."
		});
		busy = false;
	}

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

{#if poll && finalized}
	<div
		class="mx-[18px] mt-3 flex items-center justify-between gap-3 rounded-xl px-3.5 py-3"
		style="background:var(--color-accent)"
	>
		<div class="flex flex-col gap-0.5">
			<span class="text-[11.5px] font-semibold text-white">✓ Locked in</span>
			<span class="text-[14px] font-semibold text-white">{finalizedLabel}</span>
		</div>
		{#if isCreator}
			<button
				onclick={reopen}
				disabled={busy}
				class="rounded-lg border border-white/40 bg-transparent px-3 py-1.5 text-[11.5px] font-semibold text-white"
				style="cursor:{busy ? 'default' : 'pointer'}"
			>
				Reopen
			</button>
		{/if}
	</div>
{/if}

{#if poll && mine}
	<div class="flex justify-end px-[22px] pt-2">
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

{#if poll && isCreator && !finalized}
	<div
		class="mx-[18px] mt-3 flex flex-col gap-2.5 rounded-xl border p-3.5"
		style="border-color:var(--color-line)"
	>
		<span class="text-[12px] font-semibold text-ink">Lock in a date</span>
		<div class="flex gap-3">
			<label class="flex flex-1 flex-col gap-1.5">
				<span class="text-[11px] font-semibold text-subtext">From</span>
				<input
					type="date"
					bind:value={finStart}
					min={poll.rangeStart}
					max={poll.rangeEnd}
					class="rounded-[10px] border px-2.5 py-2 text-[13px] text-ink"
					style="border-color:var(--color-line)"
				/>
			</label>
			<label class="flex flex-1 flex-col gap-1.5">
				<span class="text-[11px] font-semibold text-subtext">To</span>
				<input
					type="date"
					bind:value={finEnd}
					min={finStart || poll.rangeStart}
					max={poll.rangeEnd}
					class="rounded-[10px] border px-2.5 py-2 text-[13px] text-ink"
					style="border-color:var(--color-line)"
				/>
			</label>
		</div>
		{#if finBackwards}
			<p class="m-0 text-[11px] leading-relaxed" style="color:var(--color-danger)">
				The end date can't be before the start date.
			</p>
		{/if}
		<button
			onclick={finalize}
			disabled={busy || finBackwards || !finStart || !finEnd}
			class="rounded-[10px] border-none py-2.5 text-[13px] font-semibold text-white"
			style="background:{busy || finBackwards || !finStart || !finEnd
				? 'var(--color-faint)'
				: 'var(--color-accent)'}; cursor:{busy || finBackwards || !finStart || !finEnd
				? 'default'
				: 'pointer'}"
		>
			{busy ? 'Locking in…' : 'Lock in these dates'}
		</button>
		<p class="m-0 text-[11px] leading-relaxed text-muted-2">
			Prefilled with the best overlap. Everyone sees the locked-in date — you can change or reopen
			it anytime.
		</p>
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
