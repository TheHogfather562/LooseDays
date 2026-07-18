<script lang="ts">
	import type { Snippet } from 'svelte';

	let { children, tabbar }: { children: Snippet; tabbar?: Snippet } = $props();
</script>

<div class="flex h-dvh justify-center bg-page">
	<div
		class="relative flex h-full w-full max-w-[480px] flex-col bg-card shadow-[0_0_60px_rgba(0,0,0,0.06)]"
	>
		<div class="flex-1 overflow-y-auto pb-5">
			{@render children()}
		</div>
		{#if tabbar}
			{@render tabbar()}
		{/if}
	</div>
</div>
