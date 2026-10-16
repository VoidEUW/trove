<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** The title of the screen, already translated with `t()`. */
		title: string;
		/** Icon buttons, selects and separators on the right. */
		toolbar?: Snippet;
	}

	let { title, toolbar }: Props = $props();
</script>

<header class="topbar look-open">
	<h1>{title}</h1>
	{#if toolbar}<div class="toolbar">{@render toolbar()}</div>{/if}
</header>

<style>
	.topbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: calc(var(--space) * 3);
		min-height: calc(var(--space) * 14);
		padding: 0 calc(var(--space) * 6);
		border-bottom: var(--border-width) solid var(--look-border);
		background: var(--ground);
		color: var(--text);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
	}

	h1 {
		margin: 0;
		font-family: var(--font-display);
		font-size: calc(var(--body-size) * 1.4);
		font-weight: var(--display-weight);
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: calc(var(--space) * 2);
	}
</style>
