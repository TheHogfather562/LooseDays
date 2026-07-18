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
		onDayClick,
		onSwipe
	}: {
		cells: GridCell[];
		cellSize?: number;
		onDayClick?: (date: string) => void;
		onSwipe?: (direction: 'left' | 'right') => void;
	} = $props();

	// Swipe-to-change-month: only fires once a horizontal drag clears both an
	// absolute distance and a ratio over vertical movement, so a vertical page
	// scroll started on the grid is never mistaken for a swipe.
	const SWIPE_THRESHOLD = 40;
	let touchStartX = 0;
	let touchStartY = 0;

	function onTouchStart(e: TouchEvent) {
		touchStartX = e.touches[0].clientX;
		touchStartY = e.touches[0].clientY;
	}

	function onTouchEnd(e: TouchEvent) {
		if (!onSwipe) return;
		const dx = e.changedTouches[0].clientX - touchStartX;
		const dy = e.changedTouches[0].clientY - touchStartY;
		if (Math.abs(dx) < SWIPE_THRESHOLD || Math.abs(dx) < Math.abs(dy)) return;
		onSwipe(dx < 0 ? 'left' : 'right');
	}
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div ontouchstart={onTouchStart} ontouchend={onTouchEnd}>
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
</div>
