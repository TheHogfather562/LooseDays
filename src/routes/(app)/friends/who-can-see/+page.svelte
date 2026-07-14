<script lang="ts">
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import { db } from '$lib/db.svelte';
	import { api } from '$lib/api';
</script>

<BackHeader title="Who can see my calendar" href={resolve('/friends')} />

<div class="flex flex-col gap-2.5 px-[18px] pt-4">
	{#each db.standingAccess as row (row.id)}
		<div
			class="flex items-center justify-between rounded-[14px] border bg-white px-3.5 py-3"
			style="border-color:var(--color-line)"
		>
			<div>
				<div class="text-[13.5px] font-semibold text-ink">{row.friendName}</div>
				<div class="text-[11.5px] text-muted">
					{row.detailLevel === 'full' ? 'Full status' : 'Overlap only'}
				</div>
			</div>
			<button
				onclick={() => api.revokeStandingAccess(row.id)}
				class="cursor-pointer rounded-lg border bg-white px-3 py-1.5 text-xs font-semibold"
				style="border-color:var(--color-line); color:var(--color-danger)"
			>
				Revoke
			</button>
		</div>
	{/each}
	{#if db.standingAccess.length === 0}
		<p class="py-7 text-center text-[12.5px] text-muted">Nobody has standing access yet</p>
	{/if}
</div>
