<script lang="ts">
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import { api } from '$lib/api';
	import { smsLink, whatsappLink } from '$lib/links';
	import { contactPickerSupported, pickContacts } from '$lib/contactPicker';
	import type { Contact } from '$lib/types';

	let pasteInput = $state('');
	let contacts = $state<Contact[]>([]);
	let checked = $state(false);

	let importing = $state(false);
	let importError = $state('');

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

	let inviteEmail = $state('');
	let inviteSent = $state(false);

	const inviteMsg = `Join me on Loose Days — a calendar for finding time with friends: ${
		typeof window !== 'undefined' ? window.location.origin : ''
	}/signin`;

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

	async function addFriend(contact: Contact) {
		if (!contact.userId) return;
		await api.addFriend(contact.userId);
		contacts = contacts.map((c) => (c.id === contact.id ? { ...c, alreadyFriend: true } : c));
	}

	async function sendInvite() {
		if (!inviteEmail.includes('@')) return;
		await api.inviteEmail(inviteEmail);
		inviteSent = true;
	}
</script>

<BackHeader title="Add a friend" href={resolve('/friends')} />

<div class="px-[22px] pt-2 pb-1">
	<p class="m-0 text-[12.5px] leading-relaxed text-subtext-2">
		Friends match you by phone number — check your contacts against Loose Days.
	</p>
</div>

<div class="flex flex-col gap-2 px-[22px] pt-4">
	{#if contactPickerSupported()}
		<button
			onclick={importFromContacts}
			disabled={importing}
			class="cursor-pointer self-start rounded-[10px] border-none bg-accent px-4 py-2 text-[12.5px] font-semibold text-white"
			style="cursor:{importing ? 'default' : 'pointer'}"
		>
			{importing ? 'Importing…' : 'Import from phone contacts'}
		</button>
		{#if importError}
			<p class="m-0 text-[11.5px] leading-relaxed" style="color:var(--color-danger)">
				{importError}
			</p>
		{/if}
		<span class="text-xs font-semibold text-subtext">Or paste contacts (one per line: name, phone)</span
		>
	{:else}
		<span class="text-xs font-semibold text-subtext">Paste contacts (one per line: name, phone)</span
		>
	{/if}
	<textarea
		placeholder={'Priya Shah, +1 415-555-0101\nDana Ruiz, +1 415-555-0199'}
		bind:value={pasteInput}
		class="h-20 w-full resize-none rounded-[10px] border bg-white px-3 py-2.5 text-[13px] text-ink-soft"
		style="border-color:var(--color-line)"
	></textarea>
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
		{#if contacts.length === 0}
			<p class="py-3 text-center text-[12.5px] text-muted">No contacts matched</p>
		{/if}
	</div>
{/if}

<div class="flex flex-col gap-2 px-[22px] pt-5 pb-6">
	<span class="text-xs font-semibold text-subtext">Or invite someone by email</span>
	<p class="m-0 text-[11px] leading-relaxed text-muted-2">
		Sign-up is invite-only — this lets them request their own sign-in link.
	</p>
	<div class="flex gap-2">
		<input
			type="email"
			placeholder="friend@example.com"
			bind:value={inviteEmail}
			class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
			style="border-color:var(--color-line)"
		/>
		<button
			onclick={sendInvite}
			class="cursor-pointer rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
		>
			{inviteSent ? 'Invited ✓' : 'Invite'}
		</button>
	</div>
</div>
