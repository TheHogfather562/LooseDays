<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';

	const tabs = [
		{ key: 'calendar', label: 'Calendar', href: resolve('/calendar') },
		{ key: 'friends', label: 'Friends', href: resolve('/friends') },
		{ key: 'polls', label: 'Polls', href: resolve('/polls') }
	];

	function isActive(key: string) {
		const path = page.url.pathname;
		return path === `/${key}` || path.startsWith(`/${key}/`);
	}
</script>

<div class="flex border-t border-line bg-white pt-3 pb-5">
	{#each tabs as tab, i (tab.key)}
		{@const active = isActive(tab.key)}
		<a
			href={tab.href}
			class="flex flex-1 flex-col items-center gap-[5px] py-1 text-inherit no-underline"
		>
			{#if i === 0}
				<span
					class="h-[9px] w-[9px] rounded-full"
					style="background:{active ? 'var(--color-accent)' : 'var(--color-tab-inactive)'}"
				></span>
			{:else if i === 1}
				<span
					class="h-[9px] w-[9px] rounded-[2px]"
					style="background:{active ? 'var(--color-accent)' : 'var(--color-tab-inactive)'}"
				></span>
			{:else}
				<span
					class="h-2 w-2 rotate-45"
					style="background:{active ? 'var(--color-accent)' : 'var(--color-tab-inactive)'}"
				></span>
			{/if}
			<span
				class="text-[10.5px]"
				style="color:{active ? 'var(--color-ink)' : 'var(--color-muted)'}; font-weight:{active
					? 600
					: 400}"
			>
				{tab.label}
			</span>
		</a>
	{/each}
</div>
