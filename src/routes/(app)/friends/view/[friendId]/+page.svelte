<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import CalendarMonthGrid from '$lib/components/CalendarMonthGrid.svelte';
	import StatusDot from '$lib/components/StatusDot.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';
	import { dateStr, mondayFirstWeekday } from '$lib/format';
	import type { Friend } from '$lib/types';

	let friends = $state<Friend[]>([]);
	let friendCal = $state<Record<string, string>>({});

	const friendId = $derived(page.params.friendId!);

	$effect(() => {
		api.getFriends().then((f) => (friends = f));
	});
	$effect(() => {
		api.getFriendCalendar(friendId).then((c) => (friendCal = c));
	});

	const friend = $derived(friends.find((f) => f.id === friendId));
	const access = $derived(db.friendAccess[friendId]);

	let monthOffset = $state(0);

	type CellStatus = 'free' | 'busy' | 'maybe' | 'mutual' | null;

	function overlapAt(theirStatus: string | undefined, ds: string) {
		const myStatus = db.calStatuses[ds]?.status ?? null;
		return theirStatus === 'free' && myStatus === 'free';
	}

	const monthLabel = $derived.by(() => {
		const now = new Date();
		const base = new Date(now.getFullYear(), now.getMonth() + monthOffset, 1);
		return base.toLocaleDateString('en-US', { month: 'long', year: 'numeric' });
	});

	const grid = $derived.by(() => {
		const now = new Date();
		const base = new Date(now.getFullYear(), now.getMonth() + monthOffset, 1);
		const year = base.getFullYear();
		const month = base.getMonth();
		const daysInMonth = new Date(year, month + 1, 0).getDate();
		const startWeekday = mondayFirstWeekday(new Date(year, month, 1));
		const cells: { date: string | null; day: string; status: CellStatus; overlap: boolean }[] = [];
		for (let i = 0; i < startWeekday; i++)
			cells.push({ date: null, day: '', status: null, overlap: false });
		for (let day = 1; day <= daysInMonth; day++) {
			const ds = dateStr(year, month, day);
			const theirStatus = friendCal[ds] as 'free' | 'busy' | 'maybe' | undefined;
			const overlap = overlapAt(theirStatus, ds);
			let status: CellStatus;
			if (access?.level === 'full') {
				status = theirStatus ?? null;
			} else {
				status = overlap ? 'mutual' : null;
			}
			cells.push({ date: ds, day: String(day), status, overlap });
		}
		while (cells.length % 7 !== 0)
			cells.push({ date: null, day: '', status: null, overlap: false });
		return cells;
	});

	const legend = $derived(
		access?.level === 'full'
			? [
					{ status: 'free' as CellStatus, label: 'free' },
					{ status: 'maybe' as CellStatus, label: 'maybe' },
					{ status: 'busy' as CellStatus, label: 'busy' },
					{ status: null as CellStatus, label: 'not set' }
				]
			: [
					{ status: 'mutual' as CellStatus, label: 'both free' },
					{ status: null as CellStatus, label: 'unavailable / unknown' }
				]
	);
</script>

<BackHeader
	title={friend?.displayName ?? ''}
	href={resolve('/friends')}
	subtitle={access ? (access.level === 'full' ? 'Full access' : 'Overlap only') : ''}
/>
<p class="mx-[22px] mt-3 mb-3.5 text-xs leading-relaxed text-subtext-2">
	{access?.level === 'full'
		? "You can see their full status and notes. Days you're both free are highlighted so it's easy to spot when you can meet."
		: "Only days you're both free are highlighted — their busy or maybe days stay private."}
</p>

<div class="mx-[22px] mb-2 flex items-center justify-between">
	<span class="text-[13px] font-semibold text-ink">{monthLabel}</span>
	<div class="flex gap-[18px]">
		<button
			onclick={() => (monthOffset -= 1)}
			class="cursor-pointer border-none bg-transparent p-1 text-base text-subtext"
		>
			‹
		</button>
		<button
			onclick={() => (monthOffset += 1)}
			class="cursor-pointer border-none bg-transparent p-1 text-base text-subtext"
		>
			›
		</button>
	</div>
</div>

<CalendarMonthGrid
	cells={grid}
	cellSize={44}
	onSwipe={(dir) => (monthOffset += dir === 'left' ? 1 : -1)}
/>

<div class="mx-[22px] mt-[18px] mb-5 flex flex-wrap gap-3.5 text-[11px] text-subtext">
	{#each legend as leg (leg.label)}
		<div class="flex items-center gap-1.5">
			<StatusDot status={leg.status} size={12} /><span>{leg.label}</span>
		</div>
	{/each}
	{#if access?.level === 'full'}
		<div class="flex items-center gap-1.5">
			<span
				class="inline-block rounded-[4px]"
				style="width:12px;height:12px;background:var(--color-overlap-bg);border:1px solid var(--color-overlap-text)"
			></span>
			<span>you're both free</span>
		</div>
	{/if}
</div>
