<script lang="ts">
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import StatusPillGroup from '$lib/components/StatusPillGroup.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';
	import { myInvitee } from '$lib/polls';
	import { withToast } from '$lib/toast.svelte';
	import { dateRangeArr, fmtRangeLabel, fmtWeekdayShort } from '$lib/format';
	import type { Availability } from '$lib/types';

	const pollId = $derived(page.params.id!);
	const poll = $derived(db.polls.find((p) => p.id === pollId));

	const days = $derived(poll ? dateRangeArr(poll.rangeStart, poll.rangeEnd) : []);

	// Prefill from an existing response so this screen doubles as "edit my
	// answer" — the backend re-response is a full replace, so what's shown here
	// is exactly what gets saved.
	let draft = $state<Record<string, Availability>>({});
	let seeded = $state(false);
	$effect(() => {
		if (seeded || !poll) return;
		const mine = myInvitee(poll);
		if (mine && poll.responses[mine.id]) draft = { ...poll.responses[mine.id] };
		seeded = true;
	});

	let submitting = $state(false);
	const submitDisabled = $derived(days.length === 0 || days.some((d) => !draft[d]) || submitting);

	function markAll(status: Availability) {
		const next: Record<string, Availability> = {};
		for (const d of days) next[d] = status;
		draft = next;
	}

	async function submit() {
		if (submitDisabled || !poll) return;
		submitting = true;
		const ok = await withToast(() => api.submitPollResponse(poll.id, draft), {
			success: 'Response saved.',
			error: "Couldn't submit your response — try again."
		});
		submitting = false;
		if (ok) await goto(resolve('/(app)/polls/[id]/results', { id: poll.id }));
	}
</script>

<BackHeader
	title={poll?.title ?? ''}
	href={resolve('/polls')}
	subtitle={poll ? fmtRangeLabel(poll.rangeStart, poll.rangeEnd) : ''}
/>
<div class="flex items-center justify-between gap-2 px-[22px] pt-3 pb-3.5">
	<p class="m-0 text-[12.5px] text-subtext-2">Mark your availability for each day</p>
	<div class="flex gap-1.5">
		<button
			onclick={() => markAll('free')}
			class="cursor-pointer rounded-full border px-2.5 py-1 text-[11px] font-semibold text-subtext"
			style="border-color:var(--color-line)"
		>
			All free
		</button>
		<button
			onclick={() => markAll('busy')}
			class="cursor-pointer rounded-full border px-2.5 py-1 text-[11px] font-semibold text-subtext"
			style="border-color:var(--color-line)"
		>
			All busy
		</button>
	</div>
</div>
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
		{submitting ? 'Submitting…' : 'Submit response'}
	</button>
</div>
