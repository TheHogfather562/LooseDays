<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import StatusPillGroup from '$lib/components/StatusPillGroup.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';
	import { dateRangeArr, fmtRangeLabel, fmtWeekdayShort } from '$lib/format';
	import type { Availability } from '$lib/types';

	const pollId = $derived(page.params.id!);
	const poll = $derived(db.polls.find((p) => p.id === pollId));

	let draft = $state<Record<string, Availability>>({});

	const days = $derived(poll ? dateRangeArr(poll.rangeStart, poll.rangeEnd) : []);
	const submitDisabled = $derived(days.length === 0 || days.some((d) => !draft[d]));

	async function submit() {
		if (submitDisabled || !poll) return;
		await api.submitPollResponse(poll.id, draft);
		await goto(resolve('/(app)/polls/[id]/results', { id: poll.id }));
	}
</script>

<BackHeader
	title={poll?.title ?? ''}
	href={resolve('/polls')}
	subtitle={poll ? fmtRangeLabel(poll.rangeStart, poll.rangeEnd) : ''}
/>
<p class="m-0 px-[22px] pt-3 pb-3.5 text-[12.5px] text-subtext-2">
	Mark your availability for each day
</p>
<div class="flex flex-col gap-2.5 px-[18px]">
	{#each days as d (d)}
		<div
			class="flex flex-col gap-2.5 rounded-[14px] border p-3.5"
			style="border-color:var(--color-line)"
		>
			<span class="text-[13px] font-semibold text-ink">{fmtWeekdayShort(d)}</span>
			<StatusPillGroup value={draft[d] ?? null} onSelect={(s) => (draft = { ...draft, [d]: s })} />
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
