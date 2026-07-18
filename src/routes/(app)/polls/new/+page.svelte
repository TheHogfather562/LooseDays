<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import InviteLinks from '$lib/components/InviteLinks.svelte';
	import { api } from '$lib/api';
	import { fmtRangeLabel } from '$lib/format';
	import { getRememberedPhone } from '$lib/phonebook';
	import type { Friend, Poll } from '$lib/types';

	let friends = $state<Friend[]>([]);
	$effect(() => {
		api.getFriends().then((f) => (friends = f));
	});

	let title = $state('');
	let note = $state('');
	let start = $state('');
	let end = $state('');
	let friendSel = $state<Record<string, boolean>>({});
	// The server never stores raw phone numbers, so — unlike before — a
	// name is required per phone invitee: it can't fall back to displaying
	// the phone digits as the name on later views of this poll.
	let phoneChips = $state<{ name: string; phone: string }[]>([]);
	let phoneNameInput = $state('');
	let phoneInput = $state('');
	let step = $state<'form' | 'sent'>('form');
	let createdPoll = $state<Poll | null>(null);

	function toggleFriend(id: string) {
		friendSel = { ...friendSel, [id]: !friendSel[id] };
	}
	function addPhoneChip() {
		const phone = phoneInput.trim();
		const name = phoneNameInput.trim();
		if (!phone || !name) return;
		phoneChips = [...phoneChips, { name, phone }];
		phoneNameInput = '';
		phoneInput = '';
	}
	function removePhoneChip(idx: number) {
		phoneChips = phoneChips.filter((_, i) => i !== idx);
	}

	const sendDisabled = $derived(
		!(title && start && end && (friends.some((f) => friendSel[f.id]) || phoneChips.length))
	);

	async function goToSend() {
		if (sendDisabled) return;
		createdPoll = await api.createPoll({
			title,
			note,
			start,
			end,
			friendIds: Object.keys(friendSel).filter((id) => friendSel[id]),
			phoneInvitees: phoneChips
		});
		step = 'sent';
	}
	function backToForm() {
		step = 'form';
		createdPoll = null;
	}

	const rangeLabel = $derived(start && end ? fmtRangeLabel(start, end) : '');
	const sendTargets = $derived(
		createdPoll
			? createdPoll.invitees
					.filter((inv) => !inv.isMe)
					.map((inv) => ({
						name: inv.name,
						// `inv.phone` is only present in the response right after
						// creation (the server echoes back what was just
						// submitted without persisting it); for friend invitees,
						// fall back to whatever this device has cached locally.
						phone: inv.phone ?? (inv.userId ? getRememberedPhone(inv.userId) : undefined) ?? '',
						token: inv.accessToken!
					}))
			: []
	);

	function inviteMsgFor(target: { token: string }) {
		const origin = typeof window !== 'undefined' ? window.location.origin : '';
		return `You're invited to "${title}" (${rangeLabel}) on Loose Days — mark your availability: ${origin}/p/${target.token}`;
	}

	async function finish() {
		await goto(resolve('/polls'));
	}
</script>

{#if step === 'form'}
	<BackHeader title="New poll" href={resolve('/polls')} />
	<div class="flex flex-col gap-4 px-[22px] pt-2">
		<label class="flex flex-col gap-1.5">
			<span class="text-xs font-semibold text-subtext">Title</span>
			<input
				type="text"
				placeholder="e.g. Cabin weekend"
				bind:value={title}
				class="rounded-[10px] border px-3 py-2.5 text-sm text-ink"
				style="border-color:var(--color-line)"
			/>
		</label>
		<div class="flex gap-3">
			<label class="flex flex-1 flex-col gap-1.5">
				<span class="text-xs font-semibold text-subtext">Start</span>
				<input
					type="date"
					bind:value={start}
					class="rounded-[10px] border px-2.5 py-2.5 text-[13px] text-ink"
					style="border-color:var(--color-line)"
				/>
			</label>
			<label class="flex flex-1 flex-col gap-1.5">
				<span class="text-xs font-semibold text-subtext">End</span>
				<input
					type="date"
					bind:value={end}
					class="rounded-[10px] border px-2.5 py-2.5 text-[13px] text-ink"
					style="border-color:var(--color-line)"
				/>
			</label>
		</div>
		<label class="flex flex-col gap-1.5">
			<span class="text-xs font-semibold text-subtext">Note (optional)</span>
			<input
				type="text"
				placeholder="e.g. need to book by Friday"
				bind:value={note}
				class="rounded-[10px] border px-3 py-2.5 text-sm text-ink"
				style="border-color:var(--color-line)"
			/>
		</label>

		<div class="flex flex-col gap-2">
			<span class="text-xs font-semibold text-subtext">Invite friends</span>
			{#each friends as f (f.id)}
				{@const sel = !!friendSel[f.id]}
				<button
					onclick={() => toggleFriend(f.id)}
					class="flex cursor-pointer items-center justify-between rounded-xl px-3.5 py-2.5 text-[13.5px]"
					style="border:1px solid {sel
						? 'var(--color-accent)'
						: 'var(--color-line)'}; background:{sel
						? 'var(--color-overlap-bg)'
						: '#fff'}; color:var(--color-ink)"
				>
					<span>{f.displayName}</span>
					<span
						class="h-4 w-4 rounded-full"
						style="border:2px solid {sel
							? 'var(--color-accent)'
							: 'var(--color-faint-2)'}; background:{sel ? 'var(--color-accent)' : 'transparent'}"
					></span>
				</button>
			{/each}
		</div>

		<div class="flex flex-col gap-2">
			<span class="text-xs font-semibold text-subtext">Or add by name + phone number</span>
			<div class="flex gap-2">
				<input
					type="text"
					placeholder="Name"
					bind:value={phoneNameInput}
					class="w-24 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
					style="border-color:var(--color-line)"
				/>
				<input
					type="tel"
					placeholder="+1 555-0100"
					bind:value={phoneInput}
					class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
					style="border-color:var(--color-line)"
				/>
				<button
					onclick={addPhoneChip}
					class="cursor-pointer rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
				>
					Add
				</button>
			</div>
			<div class="flex flex-wrap gap-2">
				{#each phoneChips as chip, i (chip.phone + i)}
					<span
						class="flex items-center gap-1.5 rounded-full bg-page px-2.5 py-1.5 text-[12.5px] text-ink-soft"
					>
						{chip.name} · {chip.phone}
						<button
							onclick={() => removePhoneChip(i)}
							class="cursor-pointer border-none bg-transparent p-0 text-[13px] text-muted"
						>
							×
						</button>
					</span>
				{/each}
			</div>
		</div>

		<button
			onclick={goToSend}
			disabled={sendDisabled}
			class="mt-1 mb-6 w-full rounded-[10px] border-none py-3 text-sm font-semibold text-white"
			style="background:{sendDisabled
				? 'var(--color-faint)'
				: 'var(--color-accent)'}; cursor:{sendDisabled ? 'default' : 'pointer'}"
		>
			Continue to invites
		</button>
	</div>
{:else}
	<div class="flex items-center gap-2.5 px-[18px] pt-[22px] pb-2.5">
		<button
			onclick={backToForm}
			class="cursor-pointer border-none bg-transparent p-1 text-[15px] text-subtext"
		>
			‹
		</button>
		<h1 class="m-0 font-display text-[19px] font-semibold text-ink">Send invites</h1>
	</div>
	<p class="m-0 px-[22px] pb-4 text-[12.5px] text-subtext-2">
		Loose Days doesn't message anyone directly — send each person a link over SMS or WhatsApp.
	</p>
	<div class="flex flex-col gap-2.5 px-[18px]">
		{#each sendTargets as t (t.token)}
			<div
				class="flex items-center justify-between rounded-[14px] border px-3.5 py-3"
				style="border-color:var(--color-line)"
			>
				<div>
					<div class="text-[13.5px] font-semibold text-ink">{t.name}</div>
					<div class="text-[11.5px] text-muted">{t.phone || 'No number on this device'}</div>
				</div>
				<InviteLinks phone={t.phone} message={inviteMsgFor(t)} />
			</div>
		{/each}
	</div>
	<div class="px-[18px] pt-5">
		<button
			onclick={finish}
			class="w-full cursor-pointer rounded-[10px] border-none bg-accent py-3 text-sm font-semibold text-white"
		>
			Done
		</button>
	</div>
{/if}
