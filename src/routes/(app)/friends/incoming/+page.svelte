<script lang="ts">
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import { db } from '$lib/db.svelte';
	import { api } from '$lib/api';
	import { fmtRangeLabel } from '$lib/format';
	import type { DetailLevel } from '$lib/types';

	let approvingId = $state<string | null>(null);
	let approvingLevel = $state<'full' | 'overlap'>('full');

	function startApprove(id: string) {
		approvingId = id;
		approvingLevel = 'full';
	}

	async function confirmApprove() {
		if (!approvingId) return;
		const level: DetailLevel = approvingLevel === 'full' ? 'full' : 'overlap_only';
		await api.approveAccessRequest(approvingId, level);
		approvingId = null;
	}

	async function deny(id: string) {
		await api.denyAccessRequest(id);
	}
</script>

<BackHeader title="Incoming requests" href={resolve('/friends')} />

<div class="flex flex-col gap-3 px-[18px] pt-4">
	{#each db.incomingRequests as req (req.id)}
		<div
			class="flex flex-col gap-2.5 rounded-[14px] border bg-white p-3.5"
			style="border-color:var(--color-line)"
		>
			<div>
				<div class="text-[13.5px] font-semibold text-ink">{req.requesterName}</div>
				<div class="text-[11.5px] text-muted">
					{req.scope === 'standing'
						? 'Wants ongoing access'
						: `Wants access ${fmtRangeLabel(req.rangeStart!, req.rangeEnd!)}`}
				</div>
			</div>
			{#if approvingId === req.id}
				<div class="flex flex-col gap-2">
					<span class="text-[11.5px] text-subtext">Reveal level</span>
					<div class="flex gap-2">
						<button
							onclick={() => (approvingLevel = 'full')}
							class="flex-1 cursor-pointer rounded-[10px] py-2.5 text-[12.5px] font-semibold"
							style={approvingLevel === 'full'
								? 'border:none;background:var(--color-accent);color:#fff'
								: 'border:1px solid var(--color-line);background:#fff;color:var(--color-ink)'}
						>
							Full status
						</button>
						<button
							onclick={() => (approvingLevel = 'overlap')}
							class="flex-1 cursor-pointer rounded-[10px] py-2.5 text-[12.5px] font-semibold"
							style={approvingLevel === 'overlap'
								? 'border:none;background:var(--color-accent);color:#fff'
								: 'border:1px solid var(--color-line);background:#fff;color:var(--color-ink)'}
						>
							Overlap only
						</button>
					</div>
					<button
						onclick={confirmApprove}
						class="cursor-pointer rounded-[10px] border-none bg-accent py-2.5 text-[13px] font-semibold text-white"
					>
						Confirm approval
					</button>
				</div>
			{:else}
				<div class="flex gap-2">
					<button
						onclick={() => startApprove(req.id)}
						class="flex-1 cursor-pointer rounded-[10px] border-none bg-accent py-2.5 text-[12.5px] font-semibold text-white"
					>
						Approve
					</button>
					<button
						onclick={() => deny(req.id)}
						class="flex-1 cursor-pointer rounded-[10px] border bg-white py-2.5 text-[12.5px] font-semibold text-subtext"
						style="border-color:var(--color-line)"
					>
						Deny
					</button>
				</div>
			{/if}
		</div>
	{/each}
	{#if db.incomingRequests.length === 0}
		<p class="py-7 text-center text-[12.5px] text-muted">No pending requests</p>
	{/if}
</div>
