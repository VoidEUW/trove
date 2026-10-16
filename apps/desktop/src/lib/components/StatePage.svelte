<script lang="ts">
	import type { Component } from 'svelte';
	import Inbox from '@lucide/svelte/icons/inbox';
	import SearchX from '@lucide/svelte/icons/search-x';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import WifiOff from '@lucide/svelte/icons/wifi-off';
	import Button from './Button.svelte';

	type Kind = 'empty' | 'error' | 'offline' | 'not-found';

	interface Action {
		/** The button text, already translated with `t()`. */
		label: string;
		onclick: () => void;
	}

	interface Props {
		kind?: Kind;
		/** What happened, in a few words, already translated with `t()`. */
		title: string;
		/** The one thing to do next. */
		description?: string;
		/** The main action: it leads the page. */
		primary?: Action;
		secondary?: Action;
		/** Technical details, folded away until asked for. */
		details?: string;
		/** The label of the folded technical details. */
		detailsLabel?: string;
	}

	let {
		kind = 'empty',
		title,
		description,
		primary,
		secondary,
		details,
		detailsLabel = ''
	}: Props = $props();

	const icons: Record<Kind, Component> = {
		empty: Inbox,
		error: TriangleAlert,
		offline: WifiOff,
		'not-found': SearchX
	};
	const Icon = $derived(icons[kind]);
</script>

<section class="state-page look-soft {kind}" aria-live={kind === 'error' ? 'polite' : undefined}>
	<span class="icon" aria-hidden="true"><Icon /></span>
	<h2>{title}</h2>
	{#if description}<p>{description}</p>{/if}
	{#if primary || secondary}
		<div class="actions">
			{#if secondary}<Button onclick={secondary.onclick}>{secondary.label}</Button>{/if}
			{#if primary}<Button variant="primary" onclick={primary.onclick}>{primary.label}</Button>{/if}
		</div>
	{/if}
	{#if details}
		<details>
			<summary>{detailsLabel}</summary>
			<pre>{details}</pre>
		</details>
	{/if}
</section>

<style>
	.state-page {
		display: grid;
		justify-items: center;
		gap: calc(var(--space) * 2);
		padding: calc(var(--space) * 12) calc(var(--space) * 6);
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		background: var(--surface);
		box-shadow: var(--look-shadow);
		color: var(--text);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
		text-align: center;
		/* page.enter: fade */
		animation: page-enter var(--motion-page-enter);
	}

	.icon {
		display: inline-flex;
		color: var(--text-muted);
	}

	.error .icon {
		color: var(--danger);
	}

	.icon :global(svg) {
		width: calc(var(--space) * 8);
		height: calc(var(--space) * 8);
		stroke-width: var(--icon-stroke);
	}

	h2 {
		margin: calc(var(--space) * 1) 0 0;
		font-family: var(--font-display);
		font-size: calc(var(--body-size) * 1.4);
		font-weight: var(--display-weight);
	}

	p {
		max-width: 46ch;
		margin: 0;
		color: var(--text-muted);
		font-family: var(--font-body);
		font-size: var(--body-size);
	}

	.actions {
		display: flex;
		gap: calc(var(--space) * 2);
		margin-top: calc(var(--space) * 3);
	}

	details {
		margin-top: calc(var(--space) * 4);
		color: var(--text-muted);
	}

	summary {
		cursor: pointer;
	}

	summary:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: var(--focus-width);
	}

	pre {
		max-width: 60ch;
		margin: calc(var(--space) * 2) 0 0;
		overflow: auto;
		font-family: var(--font-mono);
		font-size: calc(var(--ui-size) * 0.86);
		text-align: start;
		white-space: pre-wrap;
	}

	@keyframes page-enter {
		from {
			opacity: 0;
		}
	}
</style>
