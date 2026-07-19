<script lang="ts">
	import { toast, dismiss } from '$lib/toast.svelte';
</script>

<!-- Anchored to the phone-frame column (AppShell is `relative`), floating just
     above the tab bar so confirmations and errors don't shift layout. -->
<div
	class="pointer-events-none absolute inset-x-0 bottom-[86px] z-50 flex flex-col items-center gap-2 px-[18px]"
>
	{#each toast.items as t (t.id)}
		<button
			onclick={() => dismiss(t.id)}
			class="pointer-events-auto w-full max-w-[420px] cursor-pointer rounded-[12px] border px-3.5 py-3 text-left text-[12.5px] font-semibold shadow-[0_6px_20px_rgba(0,0,0,0.12)]"
			style={t.kind === 'error'
				? 'border-color:var(--color-busy); background:#fff; color:var(--color-busy)'
				: 'border-color:var(--color-accent); background:var(--color-accent); color:#fff'}
		>
			{t.message}
		</button>
	{/each}
</div>
