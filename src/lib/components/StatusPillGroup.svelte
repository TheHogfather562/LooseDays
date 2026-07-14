<script lang="ts">
	import type { Availability } from '$lib/types';

	let {
		value,
		onSelect
	}: { value: Availability | null; onSelect: (status: Availability) => void } = $props();

	const options: Availability[] = ['free', 'maybe', 'busy'];
	const colorVar: Record<Availability, string> = {
		free: 'var(--color-free)',
		maybe: 'var(--color-maybe)',
		busy: 'var(--color-busy)'
	};
</script>

<div class="flex gap-2">
	{#each options as opt (opt)}
		<button
			onclick={() => onSelect(opt)}
			class="flex-1 cursor-pointer rounded-[10px] py-[9px] text-[12.5px] font-semibold"
			style={value === opt
				? `border:none; background:${colorVar[opt]}; color:#fff`
				: `border:1px solid var(--color-line); background:#fff; color:var(--color-ink)`}
		>
			{opt.charAt(0).toUpperCase() + opt.slice(1)}
		</button>
	{/each}
</div>
