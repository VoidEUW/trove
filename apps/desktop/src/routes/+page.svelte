<script lang="ts">
	import { onMount } from 'svelte';
	import { errorMessage, getAppInfo } from '$lib/api';
	import { t } from '$lib/i18n/index.svelte';

	let status = $state('');

	onMount(async () => {
		try {
			status = t('home.version', { version: (await getAppInfo()).version });
		} catch (error) {
			status = errorMessage(error);
		}
	});
</script>

<main>
	<h1>{t('app.title')}</h1>
	<p class="tagline">{t('home.tagline')}</p>
	{#if status}
		<p class="status">{status}</p>
	{/if}
</main>

<style>
	main {
		max-width: 68ch;
		margin: 0 auto;
		padding: calc(var(--space) * 20) calc(var(--space) * 6);
	}

	h1 {
		margin: 0;
		font-family: var(--font-display);
		font-weight: var(--display-weight);
		font-size: 3rem;
	}

	.tagline,
	.status {
		margin: calc(var(--space) * 2) 0 0;
		color: var(--text-muted);
		font-family: var(--font-body);
		font-size: var(--body-size);
	}
</style>
