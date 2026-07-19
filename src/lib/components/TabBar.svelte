<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { db } from '$lib/db.svelte';
	import { myInvitee } from '$lib/polls';

	// Count the things that need the user's attention, so the relevant tab can
	// carry a badge instead of the user having to open each screen to find out.
	const friendsBadge = $derived(db.incomingRequests.length);
	const pollsBadge = $derived(db.polls.filter((p) => myInvitee(p)?.status === 'invited').length);

	const tabs = [
		{ key: 'calendar', label: 'Calendar', href: resolve('/calendar'), badge: 0 },
		{ key: 'friends', label: 'Friends', href: resolve('/friends'), badge: 0 },
		{ key: 'polls', label: 'Polls', href: resolve('/polls'), badge: 0 }
	];

	const badgeFor = $derived((key: string) =>
		key === 'friends' ? friendsBadge : key === 'polls' ? pollsBadge : 0
	);

	function isActive(key: string) {
		const path = page.url.pathname;
		return path === `/${key}` || path.startsWith(`/${key}/`);
	}
</script>

<div class="flex border-t border-line bg-white pt-3 pb-5">
	{#each tabs as tab, i (tab.key)}
		{@const active = isActive(tab.key)}
		{@const badge = badgeFor(tab.key)}
		<a
			href={tab.href}
			aria-current={active ? 'page' : undefined}
			class="relative flex flex-1 flex-col items-center gap-[5px] py-1 text-inherit no-underline"
		>
			<span class="relative">
				{#if i === 0}
					<span
						class="block h-[9px] w-[9px] rounded-full"
						style="background:{active ? 'var(--color-accent)' : 'var(--color-tab-inactive)'}"
					></span>
				{:else if i === 1}
					<span
						class="block h-[9px] w-[9px] rounded-[2px]"
						style="background:{active ? 'var(--color-accent)' : 'var(--color-tab-inactive)'}"
					></span>
				{:else}
					<span
						class="block h-2 w-2 rotate-45"
						style="background:{active ? 'var(--color-accent)' : 'var(--color-tab-inactive)'}"
					></span>
				{/if}
				{#if badge > 0}
					<span
						class="absolute -top-2 -right-2.5 flex h-[15px] min-w-[15px] items-center justify-center rounded-full px-1 text-[9px] font-bold text-white"
						style="background:var(--color-accent)"
						aria-label="{badge} needing attention"
					>
						{badge > 9 ? '9+' : badge}
					</span>
				{/if}
			</span>
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
