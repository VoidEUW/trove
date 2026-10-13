<script lang="ts">
	import { t } from '$lib/i18n/index.svelte';
	import { shell } from '$lib/shell.svelte';

	// The Items screen fills this in with the next issue; for now it names itself in the top bar.
	$effect(() => {
		shell.title = t('nav.items');
		return () => {
			shell.title = undefined;
			shell.toolbar = undefined;
		};
	});
</script>
