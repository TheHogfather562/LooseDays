<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import AppShell from '$lib/components/AppShell.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';
	import type { Contact } from '$lib/types';
	import { smsLink, whatsappLink } from '$lib/links';

	let contacts = $state<Contact[]>([]);

	$effect(() => {
		api.getContacts().then((c) => (contacts = c));
	});

	const inviteMsg = `Join me on Loose Days — a calendar for finding time with friends: https://loosedays.app/join`;

	async function addFriend(contactId: string) {
		await api.toggleContactAdded(contactId);
	}

	async function finish() {
		await api.completeOnboarding();
		await goto(resolve('/calendar'));
	}
</script>

<AppShell>
	<div class="px-[22px] pt-[26px] pb-1">
		<h1 class="m-0 mb-1.5 font-display text-[22px] font-semibold text-ink">Add your friends</h1>
		<p class="m-0 text-[12.5px] leading-relaxed text-subtext-2">
			We checked your contacts against Loose Days — matches shown first.
		</p>
	</div>

	<div class="flex flex-col gap-2.5 px-[18px] pt-[18px]">
		{#each contacts as c (c.id)}
			{@const added = !!db.onboardingAdded[c.id]}
			<div
				class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
				style="border-color:var(--color-line)"
			>
				<div>
					<div class="text-[13.5px] font-semibold text-ink">{c.name}</div>
					<div class="text-[11.5px] text-muted">
						{c.matched ? `${c.phone} · On Loose Days` : c.phone}
					</div>
				</div>
				{#if c.matched}
					<button
						onclick={() => addFriend(c.id)}
						disabled={added}
						class="rounded-lg px-3 py-1.5 text-xs font-semibold"
						style="border:{added ? 'none' : '1px solid var(--color-accent)'}; background:{added
							? 'var(--color-overlap-bg)'
							: '#fff'}; color:var(--color-accent); cursor:{added ? 'default' : 'pointer'}"
					>
						{added ? 'Added ✓' : 'Add friend'}
					</button>
				{:else}
					<div class="flex gap-1.5">
						<a
							href={smsLink(c.phone, inviteMsg)}
							rel="external"
							class="rounded-lg border px-2.5 py-[7px] text-[11.5px] font-semibold text-ink no-underline"
							style="border-color:var(--color-line)"
						>
							SMS
						</a>
						<a
							href={whatsappLink(c.phone, inviteMsg)}
							rel="external"
							class="rounded-lg border px-2.5 py-[7px] text-[11.5px] font-semibold text-ink no-underline"
							style="border-color:var(--color-line)"
						>
							WhatsApp
						</a>
					</div>
				{/if}
			</div>
		{/each}
	</div>

	<div class="px-[18px] pt-[22px] pb-6">
		<button
			onclick={finish}
			class="w-full cursor-pointer rounded-[10px] border-none bg-accent py-3.5 text-sm font-semibold text-white"
		>
			Continue to Loose Days
		</button>
	</div>
</AppShell>
