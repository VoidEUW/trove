<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import Package from '@lucide/svelte/icons/package';
	import Sidebar from '#lib/components/Sidebar.svelte';
	import SidebarEntry from '#lib/components/SidebarEntry.svelte';
	import Topbar from '#lib/components/Topbar.svelte';
	import { t } from '#lib/i18n/index.svelte.js';
	import { shell } from '#lib/shell.svelte.js';

	let { children } = $props();

	let collapsed = $state(false);
	let width = $state(52);
</script>

<div class="shell">
	<Sidebar
		label={t('nav.label')}
		title={t('app.title')}
		bind:collapsed
		bind:width
		collapseLabel={t('sidebar.collapse')}
		expandLabel={t('sidebar.expand')}
		resizeLabel={t('sidebar.resize')}
	>
		<SidebarEntry href="/items" label={t('nav.items')} active={page.url.pathname.startsWith('/items')}>
			{#snippet icon()}<Package />{/snippet}
		</SidebarEntry>
	</Sidebar>
	<div class="main">
		<Topbar title={shell.title ?? t('app.title')} toolbar={shell.toolbar} />
		<main>{@render children()}</main>
	</div>
</div>

<style>
	.shell {
		display: flex;
		gap: calc(var(--space) * 2);
		height: 100%;
		padding: calc(var(--space) * 2);
	}

	.main {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
	}

	main {
		flex: 1;
		overflow: auto;
		padding: calc(var(--space) * 6);
	}
</style>
