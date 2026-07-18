<script lang="ts">
	import { resolve } from '$app/paths';
	import BackHeader from '$lib/components/BackHeader.svelte';
	import ContactFinder from '$lib/components/ContactFinder.svelte';
	import PhoneFinder from '$lib/components/PhoneFinder.svelte';
	import EmailFinder from '$lib/components/EmailFinder.svelte';
</script>

<BackHeader title="Add a friend" href={resolve('/friends')} />

<div class="px-[22px] pt-2 pb-1">
	<p class="m-0 text-[12.5px] leading-relaxed text-subtext-2">
		Friends match you by phone number — check your contacts against Loose Days.
	</p>
</div>

<ContactFinder />

<PhoneFinder />

<div class="pb-6">
	<EmailFinder />
</div>
