<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import { api } from '$lib/api';
	import { db } from '$lib/db.svelte';
	import { passkeysSupported, registerPasskey } from '$lib/webauthn';
	import type { Passkey } from '$lib/types';

	function fmtDateTime(iso: string): string {
		return new Date(iso).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	}

	let passkeys = $state<Passkey[]>([]);
	let addPending = $state(false);
	let addError = $state('');

	$effect(() => {
		api.listPasskeys().then((p) => (passkeys = p));
	});

	async function addPasskey() {
		addError = '';
		addPending = true;
		try {
			await registerPasskey();
			passkeys = await api.listPasskeys();
		} catch {
			addError = "Couldn't add that passkey — try again.";
		} finally {
			addPending = false;
		}
	}

	async function removePasskey(id: string) {
		passkeys = passkeys.filter((p) => p.id !== id);
		await api.removePasskey(id);
	}

	async function signOut() {
		await api.logout();
		db.session = { checked: true, signedIn: false, onboarded: false };
		db.currentUser = null;
		await goto(resolve('/signin'));
	}
</script>

<BackHeader title="Account" href={resolve('/friends')} subtitle={db.currentUser?.email} />

<div class="flex flex-col gap-2.5 px-[18px] pt-4">
	<div class="text-[12.5px] font-semibold text-subtext">Passkeys</div>

	{#each passkeys as p (p.id)}
		<div
			class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
			style="border-color:var(--color-line)"
		>
			<div>
				<div class="text-[13.5px] font-semibold text-ink">{p.label}</div>
				<div class="text-[11.5px] text-muted">
					{p.lastUsedAt
						? `Last used ${fmtDateTime(p.lastUsedAt)}`
						: `Added ${fmtDateTime(p.createdAt)}`}
				</div>
			</div>
			<button
				onclick={() => removePasskey(p.id)}
				class="cursor-pointer rounded-lg border bg-white px-3 py-1.5 text-xs font-semibold"
				style="border-color:var(--color-line); color:var(--color-danger)"
			>
				Remove
			</button>
		</div>
	{/each}

	{#if passkeys.length === 0}
		<p class="py-2 text-[12.5px] text-muted">No passkeys yet.</p>
	{/if}

	{#if passkeysSupported()}
		<button
			onclick={addPasskey}
			disabled={addPending}
			class="self-start text-[13px] font-semibold text-accent"
			style="cursor:{addPending ? 'default' : 'pointer'}"
		>
			{addPending ? 'Waiting for passkey…' : '+ Add a passkey'}
		</button>
		{#if addError}
			<p class="m-0 text-[11.5px] leading-relaxed" style="color:var(--color-danger)">{addError}</p>
		{/if}
	{:else}
		<p class="m-0 text-[11.5px] leading-relaxed text-muted-2">
			Passkeys aren't supported in this browser.
		</p>
	{/if}

	<button
		onclick={signOut}
		class="mt-6 cursor-pointer self-start border-none bg-transparent p-0 text-[13px] font-semibold"
		style="color:var(--color-danger)"
	>
		Sign out
	</button>
</div>
