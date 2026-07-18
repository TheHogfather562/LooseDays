<script lang="ts">
	import { api } from '$lib/api';
	import { inviteMessage } from '$lib/invite';
	import InviteLinks from './InviteLinks.svelte';
	import type { Contact } from '$lib/types';

	// Find one friend by their phone number alone — no name required, since the
	// server matches purely on the number (see repo::friends::match_contacts).
	// This is the single-person counterpart to ContactFinder's bulk address-book
	// import, shared by onboarding and the add-friend screen. Matching hands off
	// to the existing matchContacts API with an empty name; if they're not here
	// yet we fall through to an invite, just like the email flow.
	let phone = $state('');
	let result = $state<Contact | null>(null);
	let searching = $state(false);
	let added = $state(false);

	const inviteMsg = inviteMessage();

	// Enough digits to be a plausible number without guessing at country rules —
	// the backend does the real parsing/normalization.
	const digits = $derived(phone.replace(/\D/g, ''));
	const valid = $derived(digits.length >= 6);

	function reset() {
		result = null;
		added = false;
	}

	async function search() {
		if (!valid) return;
		searching = true;
		reset();
		try {
			const [match] = await api.matchContacts([{ name: '', phone }]);
			result = match ?? null;
		} finally {
			searching = false;
		}
	}

	async function addFriend() {
		if (!result?.userId) return;
		await api.addFriend(result.userId);
		added = true;
	}
</script>

<div class="flex flex-col gap-2 px-[22px] pt-5">
	<span class="text-xs font-semibold text-subtext">Or find someone by number</span>
	<p class="m-0 text-[11px] leading-relaxed text-muted-2">
		Know their number? Look them up on Loose Days directly — no name needed. If they're not here
		yet, you can invite them.
	</p>
	<div class="flex gap-2">
		<input
			type="tel"
			placeholder="+1 415-555-0101"
			bind:value={phone}
			oninput={reset}
			class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
			style="border-color:var(--color-line)"
		/>
		<button
			onclick={search}
			disabled={searching || !valid}
			class="cursor-pointer rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
		>
			{searching ? 'Searching…' : 'Search'}
		</button>
	</div>

	{#if result}
		{#if result.matched}
			<div
				class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
				style="border-color:var(--color-line)"
			>
				<div>
					<div class="text-[13.5px] font-semibold text-ink">{result.name}</div>
					<div class="text-[11.5px] text-muted">On Loose Days</div>
				</div>
				<button
					onclick={addFriend}
					disabled={result.alreadyFriend || added}
					class="rounded-lg px-3 py-1.5 text-xs font-semibold"
					style="border:{result.alreadyFriend || added
						? 'none'
						: '1px solid var(--color-accent)'}; background:{result.alreadyFriend || added
						? 'var(--color-overlap-bg)'
						: '#fff'}; color:var(--color-accent); cursor:{result.alreadyFriend || added
						? 'default'
						: 'pointer'}"
				>
					{result.alreadyFriend || added ? 'Added ✓' : 'Add friend'}
				</button>
			</div>
		{:else}
			<div
				class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
				style="border-color:var(--color-line)"
			>
				<div>
					<div class="text-[11.5px] leading-relaxed text-muted">
						No one on Loose Days uses that number yet.
					</div>
				</div>
				<InviteLinks phone={result.phone} message={inviteMsg} />
			</div>
		{/if}
	{/if}
</div>
