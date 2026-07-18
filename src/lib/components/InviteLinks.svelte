<script lang="ts">
	import { smsLink, whatsappLink } from '$lib/links';

	// Loose Days never messages anyone directly — these hand off to the user's
	// own SMS/WhatsApp app. When we don't have a number for this person (e.g. a
	// friend whose number was never cached on this device), we show why the
	// buttons are missing instead of emitting dead, recipient-less links.
	let { phone, message }: { phone: string; message: string } = $props();
</script>

{#if phone}
	<div class="flex gap-1.5">
		<a
			href={smsLink(phone, message)}
			rel="external"
			class="rounded-lg border px-2.5 py-[7px] text-[11.5px] font-semibold text-ink no-underline"
			style="border-color:var(--color-line)"
		>
			SMS
		</a>
		<a
			href={whatsappLink(phone, message)}
			rel="external"
			class="rounded-lg border px-2.5 py-[7px] text-[11.5px] font-semibold text-ink no-underline"
			style="border-color:var(--color-line)"
		>
			WhatsApp
		</a>
	</div>
{:else}
	<span class="text-[11px] text-muted-2">No number on this device</span>
{/if}
