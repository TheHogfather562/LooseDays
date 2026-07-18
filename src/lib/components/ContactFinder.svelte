<script lang="ts">
	import { api } from '$lib/api';
	import { inviteMessage } from '$lib/invite';
	import { bulkSmsLink } from '$lib/links';
	import { contactPickerSupported, pickContacts } from '$lib/contactPicker';
	import type { Contact } from '$lib/types';

	// Self-contained "find friends by number" flow, shared by onboarding and the
	// add-friend screen so the two can't drift apart. Search for someone by their
	// number (or pull a batch straight from the phone's contact picker), building
	// up a list as you go. People already on Loose Days can be added as friends
	// inline; everyone who isn't here yet gets queued so you can fire off all the
	// invites in a single SMS hand-off at the end.
	let searchInput = $state('');
	let list = $state<Contact[]>([]);
	let searching = $state(false);
	let importing = $state(false);
	let error = $state('');

	const inviteMsg = inviteMessage();

	// Enough digits to be a plausible number without guessing at country rules —
	// the backend does the real parsing/normalization.
	const digits = $derived(searchInput.replace(/\D/g, ''));
	const valid = $derived(digits.length >= 6);

	const invitees = $derived(list.filter((c) => !c.matched));

	function keyOf(phone: string): string {
		return phone.replace(/\D/g, '');
	}

	// Single-call matchContacts reuses the backend id `c0_…` for every lookup, so
	// mint our own stable key for the {#each} and de-dupe by number / account.
	function merge(found: Contact[]) {
		for (const c of found) {
			const dup = list.some(
				(existing) =>
					keyOf(existing.phone) === keyOf(c.phone) ||
					(c.userId != null && existing.userId === c.userId)
			);
			if (dup) continue;
			list.push({ ...c, id: crypto.randomUUID() });
		}
	}

	async function addByNumber() {
		if (!valid || searching) return;
		error = '';
		searching = true;
		try {
			const found = await api.matchContacts([{ name: '', phone: searchInput.trim() }]);
			merge(found);
			searchInput = '';
		} catch (err) {
			console.error('number lookup failed', err);
			error = "Couldn't look that number up — try again.";
		} finally {
			searching = false;
		}
	}

	async function importFromContacts() {
		error = '';
		importing = true;
		try {
			const picked = await pickContacts();
			if (picked.length === 0) return;
			merge(await api.matchContacts(picked));
		} catch (err) {
			console.error('contact import failed', err);
			error = "Couldn't import contacts — try again.";
		} finally {
			importing = false;
		}
	}

	function remove(contact: Contact) {
		list = list.filter((c) => c.id !== contact.id);
	}

	async function addFriend(contact: Contact) {
		if (!contact.userId) return;
		await api.addFriend(contact.userId);
		list = list.map((c) => (c.id === contact.id ? { ...c, alreadyFriend: true } : c));
	}

	const bulkInviteHref = $derived(
		bulkSmsLink(
			invitees.map((c) => c.phone),
			inviteMsg
		)
	);
</script>

<div class="flex flex-col gap-2 px-[22px] pt-4">
	{#if contactPickerSupported()}
		<button
			onclick={importFromContacts}
			disabled={importing}
			class="self-start rounded-[10px] border-none bg-accent px-4 py-2 text-[12.5px] font-semibold text-white"
			style="cursor:{importing ? 'default' : 'pointer'}"
		>
			{importing ? 'Importing…' : 'Import from phone contacts'}
		</button>
		<span class="text-xs font-semibold text-subtext">Or add someone by number</span>
	{:else}
		<span class="text-xs font-semibold text-subtext">Add someone by number</span>
	{/if}
	<div class="flex gap-2">
		<input
			type="tel"
			placeholder="+1 415-555-0101"
			bind:value={searchInput}
			onkeydown={(e) => e.key === 'Enter' && addByNumber()}
			class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
			style="border-color:var(--color-line)"
		/>
		<button
			onclick={addByNumber}
			disabled={searching || !valid}
			class="cursor-pointer rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
		>
			{searching ? 'Adding…' : 'Add'}
		</button>
	</div>
	{#if error}
		<p class="m-0 text-[11.5px] leading-relaxed" style="color:var(--color-danger)">
			{error}
		</p>
	{/if}
</div>

{#if list.length > 0}
	<div class="flex flex-col gap-2.5 px-[18px] pt-[18px]">
		{#each list as c (c.id)}
			<div
				class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
				style="border-color:var(--color-line)"
			>
				<div class="min-w-0">
					<div class="truncate text-[13.5px] font-semibold text-ink">{c.name}</div>
					<div class="text-[11.5px] text-muted">
						{c.matched ? `${c.phone} · On Loose Days` : `${c.phone} · Not here yet`}
					</div>
				</div>
				<div class="flex flex-shrink-0 items-center gap-2">
					{#if c.matched}
						<button
							onclick={() => addFriend(c)}
							disabled={c.alreadyFriend}
							class="rounded-lg px-3 py-1.5 text-xs font-semibold"
							style="border:{c.alreadyFriend
								? 'none'
								: '1px solid var(--color-accent)'}; background:{c.alreadyFriend
								? 'var(--color-overlap-bg)'
								: '#fff'}; color:var(--color-accent); cursor:{c.alreadyFriend
								? 'default'
								: 'pointer'}"
						>
							{c.alreadyFriend ? 'Added ✓' : 'Add friend'}
						</button>
					{:else}
						<span
							class="rounded-lg px-2.5 py-1 text-[11px] font-semibold text-muted"
							style="background:var(--color-overlap-bg)">Will invite</span
						>
					{/if}
					<button
						onclick={() => remove(c)}
						aria-label="Remove from list"
						class="cursor-pointer border-none bg-transparent px-1 text-base leading-none text-muted-2"
						>×</button
					>
				</div>
			</div>
		{/each}
	</div>

	{#if invitees.length > 0}
		<div class="flex flex-col gap-1.5 px-[22px] pt-4">
			<a
				href={bulkInviteHref}
				rel="external"
				class="self-start rounded-[10px] bg-chip px-4 py-2 text-[12.5px] font-semibold text-ink no-underline"
			>
				Send {invitees.length} invite{invitees.length === 1 ? '' : 's'}
			</a>
			<p class="m-0 text-[11px] leading-relaxed text-muted-2">
				Opens your messaging app with everyone not yet on Loose Days, ready to text.
			</p>
		</div>
	{/if}
{/if}
