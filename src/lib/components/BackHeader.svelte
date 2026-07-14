<script lang="ts">
	import type { ResolvedPathname } from '$app/types';

	let { title, href, subtitle }: { title: string; href: ResolvedPathname; subtitle?: string } =
		$props();
</script>

<div class="flex items-center gap-2.5 px-[18px] pt-[22px] pb-1">
	<a {href} class="rounded p-1 text-subtext no-underline" style="font-size:15px; line-height:1">
		‹
	</a>
	<div>
		<h1 class="m-0 font-display text-[19px] font-semibold text-ink">{title}</h1>
		{#if subtitle}
			<div class="text-[12px] text-subtext-2">{subtitle}</div>
		{/if}
	</div>
</div>
