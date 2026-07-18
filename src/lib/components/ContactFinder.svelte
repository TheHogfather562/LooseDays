<script lang="ts">
	import { api } from '$lib/api';
	import { inviteMessage } from '$lib/invite';
	import { contactPickerSupported, pickContacts } from '$lib/contactPicker';
	import InviteLinks from './InviteLinks.svelte';
	import type { Contact } from '$lib/types';

	// Self-contained "add friends from your contacts" flow, shared by onboarding
	// and the add-friend screen so the two can't drift apart. Import from the
	// phone's contact picker (where supported) or paste "name, phone" lines,
	// match against Loose Days, then add matches or hand off an invite.
	let pasteInput = $state('');
	let contacts = $state<Contact[]>([]);
	let checked = $state(false);
	let importing = $state(false);
	let importError = $state('');

	const inviteMsg = inviteMessage();

	function parseEntries(): { name: string; phone: string }[] {
		return pasteInput
			.split('\n')
			.map((line) => line.trim())
			.filter(Boolean)
			.map((line) => {
				const [name, ...rest] = line.split(',');
				return { name: name.trim(), phone: rest.join(',').trim() || name.trim() };
			});
	}

	async function checkContacts() {
		const entries = parseEntries();
		if (entries.length === 0) return;
		contacts = await api.matchContacts(entries);
		checked = true;
	}

	async function importFromContacts() {
		importError = '';
		importing = true;
		try {
			const picked = await pickContacts();
			if (picked.length === 0) return;
			contacts = await api.matchContacts(picked);
			checked = true;
		} catch (err) {
			console.error('contact import failed', err);
			importError = "Couldn't import contacts — try again.";
		} finally {
			importing = false;
		}
	}

	async function addFriend(contact: Contact) {
		if (!contact.userId) return;
		await api.addFriend(contact.userId);
		contacts = contacts.map((c) => (c.id === contact.id ? { ...c, alreadyFriend: true } : c));
	}
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
		{#if importError}
			<p class="m-0 text-[11.5px] leading-relaxed" style="color:var(--color-danger)">
				{importError}
			</p>
		{/if}
		<span class="text-xs font-semibold text-subtext"
			>Or paste contacts (one per line: name, phone)</span
		>
	{:else}
		<span class="text-xs font-semibold text-subtext"
			>Paste contacts (one per line: name, phone)</span
		>
	{/if}
	<textarea
		placeholder="Priya Shah, +1 415-555-0101&#10;Dana Ruiz, +1 415-555-0199"
		bind:value={pasteInput}
		class="h-20 w-full resize-none rounded-[10px] border bg-white px-3 py-2.5 text-[13px] text-ink-soft"
		style="border-color:var(--color-line)"></textarea>
	<button
		onclick={checkContacts}
		class="cursor-pointer self-start rounded-[10px] border-none bg-chip px-4 py-2 text-[12.5px] font-semibold text-ink"
	>
		Check contacts
	</button>
</div>

{#if checked}
	<div class="flex flex-col gap-2.5 px-[18px] pt-[18px]">
		{#each contacts as c (c.id)}
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
					<InviteLinks phone={c.phone} message={inviteMsg} />
				{/if}
			</div>
		{/each}
		{#if contacts.length === 0}
			<p class="py-3 text-center text-[12.5px] text-muted">No contacts matched</p>
		{/if}
	</div>
{/if}
