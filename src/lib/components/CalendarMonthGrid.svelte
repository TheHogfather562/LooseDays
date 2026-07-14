<script lang="ts">
	import StatusDot from './StatusDot.svelte';
	import { WEEKDAY_LABELS } from '$lib/format';

	export interface GridCell {
		date: string | null;
		day: string;
		status: 'free' | 'busy' | 'maybe' | 'mutual' | null;
		isToday?: boolean;
		isSelected?: boolean;
	}

	let {
		cells,
		cellSize = 50,
		onDayClick
	}: { cells: GridCell[]; cellSize?: number; onDayClick?: (date: string) => void } = $props();
</script>

<div class="grid grid-cols-7 px-[18px] text-center">
	{#each WEEKDAY_LABELS as wl, i (i)}
		<div class="pt-0.5 pb-2.5 text-[11px] font-medium text-muted-2">{wl}</div>
	{/each}
</div>
<div class="grid grid-cols-7 gap-y-2 px-[18px]">
	{#each cells as cell, i (cell.date ?? 'pad' + i)}
		{#if cell.date === null}
			<div style="height:{cellSize}px"></div>
		{:else if onDayClick}
			<button
				onclick={() => onDayClick(cell.date!)}
				class="flex cursor-pointer flex-col items-center justify-center gap-1 rounded-[14px] bg-transparent p-0"
				style="height:{cellSize}px; border:1.5px solid {cell.isSelected
					? 'var(--color-accent)'
					: 'transparent'}"
			>
				<span
					class="text-[13px]"
					style="color:{cell.isToday
						? 'var(--color-accent)'
						: 'var(--color-ink-soft)'}; font-weight:{cell.isToday ? 700 : 400}"
				>
					{cell.day}
				</span>
				<StatusDot status={cell.status} size={15} />
			</button>
		{:else}
			<div class="flex flex-col items-center justify-center gap-1" style="height:{cellSize}px">
				<span class="text-[13px] text-ink-soft">{cell.day}</span>
				<StatusDot status={cell.status} size={15} />
			</div>
		{/if}
	{/each}
</div>
