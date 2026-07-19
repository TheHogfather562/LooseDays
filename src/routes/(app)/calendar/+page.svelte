<script lang="ts">
	import { db } from '$lib/db.svelte';
	import { api } from '$lib/api';
	import { withToast } from '$lib/toast.svelte';
	import CalendarMonthGrid from '$lib/components/CalendarMonthGrid.svelte';
	import StatusPillGroup from '$lib/components/StatusPillGroup.svelte';
	import StatusDot from '$lib/components/StatusDot.svelte';
	import { dateStr, fmtWeekdayShort, mondayFirstWeekday, todayStr } from '$lib/format';
	import { resolve } from '$app/paths';
	import type { Availability } from '$lib/types';

	let monthOffset = $state(0);
	let selectedDate = $state<string | null>(null);
	let noteDraft = $state('');
	let savingNote = $state(false);

	const grid = $derived.by(() => {
		const now = new Date();
		const base = new Date(now.getFullYear(), now.getMonth() + monthOffset, 1);
		const y = base.getFullYear();
		const m = base.getMonth();
		const daysInMonth = new Date(y, m + 1, 0).getDate();
		const startWeekday = mondayFirstWeekday(new Date(y, m, 1));
		const today = todayStr();
		const cells: {
			date: string | null;
			day: string;
			status: Availability | null;
			isToday?: boolean;
			isSelected?: boolean;
		}[] = [];
		for (let i = 0; i < startWeekday; i++) cells.push({ date: null, day: '', status: null });
		for (let day = 1; day <= daysInMonth; day++) {
			const ds = dateStr(y, m, day);
			const entry = db.calStatuses[ds];
			cells.push({
				date: ds,
				day: String(day),
				status: entry?.status ?? null,
				isToday: ds === today,
				isSelected: ds === selectedDate
			});
		}
		while (cells.length % 7 !== 0) cells.push({ date: null, day: '', status: null });
		return {
			monthLabel: base.toLocaleDateString('en-US', { month: 'long', year: 'numeric' }),
			cells
		};
	});

	const selectedStatus = $derived(
		selectedDate ? (db.calStatuses[selectedDate]?.status ?? null) : null
	);

	function selectDay(ds: string) {
		selectedDate = ds;
		noteDraft = db.calStatuses[ds]?.note ?? '';
	}

	async function setStatus(status: Availability) {
		if (!selectedDate) return;
		// api.* updates db.calStatuses optimistically before the request settles,
		// so on failure resync from the server to undo the stale local change.
		const ok = await withToast(() => api.setCalendarDayStatus(selectedDate!, status), {
			error: "Couldn't save that — try again."
		});
		if (!ok) await api.getCalendarDays().catch(() => {});
	}

	async function clearDay() {
		if (!selectedDate) return;
		const ok = await withToast(() => api.clearCalendarDay(selectedDate!), {
			error: "Couldn't clear that day — try again."
		});
		if (ok) noteDraft = '';
		else await api.getCalendarDays().catch(() => {});
	}

	function closePanel() {
		selectedDate = null;
		noteDraft = '';
	}

	async function saveNote() {
		if (!selectedDate || savingNote) return;
		savingNote = true;
		const ok = await withToast(() => api.setCalendarDayNote(selectedDate!, noteDraft), {
			success: 'Note saved.',
			error: "Couldn't save the note — try again."
		});
		if (!ok) await api.getCalendarDays().catch(() => {});
		savingNote = false;
	}

	function prevMonth() {
		monthOffset -= 1;
		selectedDate = null;
	}
	function nextMonth() {
		monthOffset += 1;
		selectedDate = null;
	}
</script>

<div class="flex items-center justify-between px-[22px] pt-[26px] pb-1">
	<a
		href={resolve('/account')}
		aria-label="Account settings"
		class="inline-block h-[30px] w-[30px] rounded-full bg-avatar"
	></a>
	<span class="text-xs text-muted">My Calendar</span>
</div>
<div class="flex items-baseline justify-between px-[22px] pt-2.5 pb-1">
	<h1 class="m-0 font-display text-2xl font-semibold text-ink">{grid.monthLabel}</h1>
	<div class="flex gap-[18px]">
		<button
			onclick={prevMonth}
			class="cursor-pointer border-none bg-transparent p-1 text-base text-subtext"
		>
			‹
		</button>
		<button
			onclick={nextMonth}
			class="cursor-pointer border-none bg-transparent p-1 text-base text-subtext"
		>
			›
		</button>
	</div>
</div>
<p class="m-0 mb-3.5 px-[22px] text-[12.5px] text-subtext-2">Tap a day to set your status</p>

<CalendarMonthGrid
	cells={grid.cells}
	cellSize={50}
	onDayClick={selectDay}
	onSwipe={(dir) => (dir === 'left' ? nextMonth() : prevMonth())}
/>

<div class="mx-[22px] mt-[18px] mb-1.5 flex gap-3.5 text-[11px] text-subtext">
	<div class="flex items-center gap-1.5">
		<StatusDot status="free" size={12} /><span>free</span>
	</div>
	<div class="flex items-center gap-1.5">
		<StatusDot status="maybe" size={12} /><span>maybe</span>
	</div>
	<div class="flex items-center gap-1.5">
		<StatusDot status="busy" size={12} /><span>busy</span>
	</div>
</div>

{#if selectedDate}
	<div
		class="mx-[18px] mt-3.5 rounded-2xl border p-4"
		style="border-color:var(--color-line); background:var(--color-panel)"
	>
		<div class="mb-3 flex items-center justify-between">
			<span class="text-[13px] font-semibold text-ink">{fmtWeekdayShort(selectedDate)}</span>
			<div class="flex gap-3">
				<button
					onclick={clearDay}
					class="cursor-pointer border-none bg-transparent text-[13px] text-muted"
				>
					Clear
				</button>
				<button
					onclick={closePanel}
					class="cursor-pointer border-none bg-transparent text-[13px] text-muted"
				>
					Close
				</button>
			</div>
		</div>
		<div class="mb-3">
			<StatusPillGroup value={selectedStatus} onSelect={setStatus} />
		</div>
		<textarea
			placeholder="Add a short note (optional)"
			bind:value={noteDraft}
			class="h-14 w-full resize-none rounded-[10px] border bg-white px-3 py-2.5 text-[13px] text-ink-soft"
			style="border-color:var(--color-line)"></textarea>
		<button
			onclick={saveNote}
			disabled={savingNote}
			class="mt-2.5 w-full rounded-[10px] border-none bg-accent py-2.5 text-[13.5px] font-semibold text-white"
			style="cursor:{savingNote ? 'default' : 'pointer'}"
		>
			{savingNote ? 'Saving…' : 'Save note'}
		</button>
	</div>
{/if}
