<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import AppShell from '$lib/components/AppShell.svelte';
	import ContactFinder from '$lib/components/ContactFinder.svelte';
	import EmailFinder from '$lib/components/EmailFinder.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';
	import { withToast } from '$lib/toast.svelte';

	// The server only ever stores a hash of the phone number, so it can't be
	// pre-filled for editing — only whether one was already set.
	let phone = $state('');
	let phoneSaved = $state(!!db.currentUser?.phoneSet);
	let saving = $state(false);
	let finishing = $state(false);

	// Enough digits to be a plausible number without guessing at country rules —
	// the backend does the real parsing/normalization.
	const canSave = $derived(phone.replace(/\D/g, '').length >= 6 && !saving);

	// Editing the field after a save means the saved value is now stale — drop
	// the confirmed state so the button (and the Continue gate) reflect that the
	// edit hasn't been persisted yet.
	function onPhoneInput() {
		phoneSaved = false;
	}

	async function savePhone() {
		if (!canSave) return;
		saving = true;
		const ok = await withToast(() => api.setMyPhone(phone.trim()), {
			error: "Couldn't save your number — try again."
		});
		saving = false;
		if (ok) phoneSaved = true;
	}

	async function finish() {
		if (!phoneSaved || finishing) return;
		finishing = true;
		const ok = await withToast(() => api.completeOnboarding(), {
			error: "Couldn't finish setup — try again."
		});
		if (ok) {
			await goto(resolve('/calendar'));
			return;
		}
		finishing = false;
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
				oninput={onPhoneInput}
				class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
				style="border-color:var(--color-line)"
			/>
			<button
				onclick={savePhone}
				disabled={!canSave && !phoneSaved}
				class="rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
				style="cursor:{canSave || phoneSaved ? 'pointer' : 'default'}"
			>
				{saving ? 'Saving…' : phoneSaved ? 'Saved ✓' : 'Save'}
			</button>
		</div>
	</div>

	<ContactFinder />

	<EmailFinder />

	<div class="px-[18px] pt-[22px] pb-6">
		<button
			onclick={finish}
			disabled={!phoneSaved || finishing}
			class="w-full rounded-[10px] border-none py-3.5 text-sm font-semibold text-white"
			style="background:{phoneSaved
				? 'var(--color-accent)'
				: 'var(--color-faint)'}; cursor:{phoneSaved && !finishing ? 'pointer' : 'default'}"
		>
			{finishing ? 'Setting up…' : 'Continue to Loose Days'}
		</button>
	</div>
</AppShell>
