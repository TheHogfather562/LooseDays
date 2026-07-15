<script lang="ts">
	import { page } from '$app/state';
	import AppShell from '$lib/components/AppShell.svelte';
	import StatusPillGroup from '$lib/components/StatusPillGroup.svelte';
	import { api } from '$lib/api';
	import { dateRangeArr, fmtRangeLabel, fmtWeekdayShort } from '$lib/format';
	import type { Availability, Poll } from '$lib/types';

	const token = $derived(page.params.token!);
	let poll = $state<Poll | null>(null);
	let notFound = $state(false);
	let submitted = $state(false);
	let draft = $state<Record<string, Availability>>({});

	$effect(() => {
		api.getPublicPoll(token).then(
			(p) => (poll = p),
			() => (notFound = true)
		);
	});

	const days = $derived(poll ? dateRangeArr(poll.rangeStart, poll.rangeEnd) : []);
	const submitDisabled = $derived(days.length === 0 || days.some((d) => !draft[d]));

	async function submit() {
		if (submitDisabled || !poll) return;
		await api.submitPublicPollResponse(token, draft);
		submitted = true;
	}
</script>

<AppShell>
	{#if notFound}
		<p class="px-[22px] py-16 text-center text-[13px] text-muted">
			This poll link is no longer valid.
		</p>
	{:else if !poll}
		<p class="px-[22px] py-16 text-center text-[13px] text-muted">Loading…</p>
	{:else if submitted}
		<div class="flex flex-col items-center justify-center gap-2 px-[22px] py-16 text-center">
			<h1 class="m-0 font-display text-xl font-semibold text-ink">Thanks!</h1>
			<p class="m-0 text-[13px] text-subtext">
				Your availability for "{poll.title}" was sent.
			</p>
		</div>
	{:else}
		<div class="px-[22px] pt-[26px] pb-1">
			<div class="mb-4 flex items-center gap-2">
				<span class="h-2.5 w-2.5 rounded-full bg-accent"></span>
				<span class="font-display text-[13px] font-semibold tracking-[0.01em] text-ink"
					>Loose Days</span
				>
			</div>
			<h1 class="m-0 mb-1 font-display text-[21px] font-semibold text-ink">{poll.title}</h1>
			<p class="m-0 text-[12.5px] text-subtext-2">
				{fmtRangeLabel(poll.rangeStart, poll.rangeEnd)}
			</p>
		</div>
		<p class="m-0 px-[22px] pt-3 pb-3.5 text-[12.5px] text-subtext-2">
			Mark your availability for each day — no account needed.
		</p>
		<div class="flex flex-col gap-2.5 px-[18px]">
			{#each days as d (d)}
				<div
					class="flex flex-col gap-2.5 rounded-[14px] border p-3.5"
					style="border-color:var(--color-line)"
				>
					<span class="text-[13px] font-semibold text-ink">{fmtWeekdayShort(d)}</span>
					<StatusPillGroup
						value={draft[d] ?? null}
						onSelect={(s) => (draft = { ...draft, [d]: s })}
					/>
				</div>
			{/each}
		</div>
		<div class="px-[18px] pt-5">
			<button
				onclick={submit}
				disabled={submitDisabled}
				class="mb-6 w-full rounded-[10px] border-none py-3 text-sm font-semibold text-white"
				style="background:{submitDisabled
					? 'var(--color-faint)'
					: 'var(--color-accent)'}; cursor:{submitDisabled ? 'default' : 'pointer'}"
			>
				Submit response
			</button>
		</div>
	{/if}
</AppShell>
