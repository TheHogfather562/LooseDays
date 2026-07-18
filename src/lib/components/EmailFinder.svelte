<script lang="ts">
	import { api } from '$lib/api';
	import type { EmailSearchResult } from '$lib/types';

	// Find a friend already on Loose Days by email, or invite them if they're
	// not here yet (sign-up is invite-only). Searching first means we never fire
	// a pointless "invite" at someone who already has an account. Shared by
	// onboarding and the add-friend screen.
	let email = $state('');
	let result = $state<EmailSearchResult | null>(null);
	let searching = $state(false);
	let added = $state(false);
	let invited = $state(false);

	function reset() {
		result = null;
		added = false;
		invited = false;
	}

	async function search() {
		if (!email.includes('@')) return;
		searching = true;
		reset();
		try {
			result = await api.searchEmail(email);
		} finally {
			searching = false;
		}
	}

	async function addByEmail() {
		if (!result?.userId) return;
		await api.addFriend(result.userId);
		added = true;
	}

	async function invite() {
		if (!email.includes('@')) return;
		await api.inviteEmail(email);
		invited = true;
	}
</script>

<div class="flex flex-col gap-2 px-[22px] pt-5">
	<span class="text-xs font-semibold text-subtext">Or find someone by email</span>
	<p class="m-0 text-[11px] leading-relaxed text-muted-2">
		Search for a friend already on Loose Days, or invite them if they're not here yet — we'll email
		them a link to join.
	</p>
	<div class="flex gap-2">
		<input
			type="email"
			placeholder="friend@example.com"
			bind:value={email}
			oninput={reset}
			class="flex-1 rounded-[10px] border px-3 py-2.5 text-[13px] text-ink"
			style="border-color:var(--color-line)"
		/>
		<button
			onclick={search}
			disabled={searching || !email.includes('@')}
			class="cursor-pointer rounded-[10px] border-none bg-chip px-4 text-[13px] font-semibold text-ink"
		>
			{searching ? 'Searching…' : 'Search'}
		</button>
	</div>

	{#if result}
		{#if result.found}
			<div
				class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
				style="border-color:var(--color-line)"
			>
				<div>
					<div class="text-[13.5px] font-semibold text-ink">{result.displayName}</div>
					<div class="text-[11.5px] text-muted">On Loose Days</div>
				</div>
				<button
					onclick={addByEmail}
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
			<p class="m-0 text-[11.5px] leading-relaxed text-muted">
				No one on Loose Days uses that email yet.
			</p>
			<button
				onclick={invite}
				class="cursor-pointer self-start rounded-[10px] border-none bg-chip px-4 py-2 text-[12.5px] font-semibold text-ink"
			>
				{invited ? 'Invite sent ✓' : 'Invite them'}
			</button>
		{/if}
	{/if}
</div>
