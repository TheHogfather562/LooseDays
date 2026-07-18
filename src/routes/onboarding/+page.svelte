<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import AppShell from '$lib/components/AppShell.svelte';
	import ContactFinder from '$lib/components/ContactFinder.svelte';
	import EmailFinder from '$lib/components/EmailFinder.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';

	// The server only ever stores a hash of the phone number, so it can't be
	// pre-filled for editing — only whether one was already set.
	let phone = $state('');
	let phoneSaved = $state(!!db.currentUser?.phoneSet);

	async function savePhone() {
		if (!phone.trim()) return;
		await api.setMyPhone(phone.trim());
		phoneSaved = true;
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
			Friends match you by phone number — set yours, then look people up to add or invite them.
		</p>
	</div>

	<div class="flex flex-col gap-2 px-[22px] pt-[18px]">
		<span class="text-xs font-semibold text-subtext">Your phone number</span>
		<div class="flex gap-2">
			<input
				type="tel"
				placeholder="+1 555-0100"
				bind:value={phone}
				class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
				style="border-color:var(--color-line)"
			/>
			<button
				onclick={savePhone}
				class="cursor-pointer rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
			>
				{phoneSaved ? 'Saved ✓' : 'Save'}
			</button>
		</div>
	</div>

	<ContactFinder />

	<EmailFinder />

	<div class="px-[18px] pt-[22px] pb-6">
		<button
			onclick={finish}
			disabled={!phoneSaved}
			class="w-full rounded-[10px] border-none py-3.5 text-sm font-semibold text-white"
			style="background:{phoneSaved
				? 'var(--color-accent)'
				: 'var(--color-faint)'}; cursor:{phoneSaved ? 'pointer' : 'default'}"
		>
			Continue to Loose Days
		</button>
	</div>
</AppShell>
