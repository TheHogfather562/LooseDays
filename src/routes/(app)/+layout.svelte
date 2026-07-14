<script lang="ts">
	import AppShell from '$lib/components/AppShell.svelte';
	import TabBar from '$lib/components/TabBar.svelte';

	let { children } = $props();
</script>

<AppShell>
	{@render children()}
	{#snippet tabbar()}
		<TabBar />
	{/snippet}
</AppShell>
