<script lang="ts">
	type Status = 'free' | 'busy' | 'maybe' | 'mutual' | null;
	let { status, size = 15 }: { status: Status; size?: number } = $props();
	const unsetSize = 5;
	const unsetMargin = $derived((size - unsetSize) / 2);
</script>

{#if status === 'busy'}
	<span class="inline-block rounded-full bg-busy" style="width:{size}px;height:{size}px"></span>
{:else if status === 'maybe'}
	<span
		class="box-border inline-block rounded-full border-2 border-maybe"
		style="width:{size}px;height:{size}px;background:linear-gradient(90deg,var(--color-maybe) 50%,transparent 50%)"
	></span>
{:else if status === 'free'}
	<span
		class="box-border inline-block rounded-full border-2 border-free"
		style="width:{size}px;height:{size}px"
	></span>
{:else if status === 'mutual'}
	<span class="inline-block rounded-full bg-free" style="width:{size}px;height:{size}px"></span>
{:else}
	<span
		class="inline-block rounded-full bg-faint"
		style="width:{unsetSize}px;height:{unsetSize}px;margin:{unsetMargin}px"
	></span>
{/if}
