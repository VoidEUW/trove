<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		open?: boolean;
		/** The title, already translated with `t()`. */
		title: string;
		/** What the dialog is about, in a sentence. */
		description?: string;
		/** The body. */
		children?: Snippet;
		/** The footer actions: put the primary one last. */
		actions?: Snippet;
		/** For the settings layout: a sidebar on the left and the body as the pane on the right. */
		sidebar?: Snippet;
		/** Changes when the settings pane changes, so the new pane fades in (`page.enter`). */
		paneKey?: string;
		/** Called when the dialog closes, however it was closed. */
		onclose?: () => void;
	}

	let {
		open = $bindable(false),
		title,
		description,
		children,
		actions,
		sidebar,
		paneKey,
		onclose
	}: Props = $props();

	const id = $props.id();
	let dialog = $state<HTMLDialogElement>();

	// The platform's modal dialog gives the scrim, Escape, a focus trap and an inert page behind it.
	$effect(() => {
		if (!dialog) return;
		if (open && !dialog.open) dialog.showModal();
		else if (!open && dialog.open) dialog.close();
	});
</script>

<dialog
	bind:this={dialog}
	class="dialog look-float"
	class:settings={sidebar}
	aria-labelledby="{id}-title"
	aria-describedby={description ? `${id}-description` : undefined}
	onclose={() => {
		open = false;
		onclose?.();
	}}
>
	{#if sidebar}
		<aside class="sidebar">{@render sidebar()}</aside>
	{/if}
	<div class="content">
		<header>
			<h2 id="{id}-title">{title}</h2>
			{#if description}<p id="{id}-description">{description}</p>{/if}
		</header>
		{#key paneKey}
			<div class="body" class:pane={sidebar}>{@render children?.()}</div>
		{/key}
		{#if actions}
			<footer>{@render actions()}</footer>
		{/if}
	</div>
</dialog>

<style>
	.dialog {
		width: calc(var(--space) * 110);
		max-width: calc(100vw - var(--space) * 8);
		margin: auto;
		padding: 0;
		/* Visible, so a menu opened inside the dialog is not cut off at its edge. */
		overflow: visible;
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		background: var(--look-bg);
		box-shadow: var(--look-shadow);
		color: var(--text);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
	}

	.dialog[open] {
		/* modal.enter: zoom */
		animation: dialog-enter var(--motion-modal-enter);
	}

	.dialog::backdrop {
		background: var(--scrim);
	}

	.dialog.settings {
		width: calc(var(--space) * 190);
	}

	/* Only while open: a `display` here would show a closed dialog. */
	.dialog.settings[open] {
		display: grid;
		grid-template-columns: calc(var(--space) * 50) 1fr;
	}

	.sidebar {
		padding: calc(var(--space) * 4);
		border-inline-end: var(--border-width) solid var(--line);
	}

	.content {
		display: grid;
		align-content: start;
		gap: calc(var(--space) * 4);
		padding: calc(var(--space) * 5);
	}

	h2 {
		margin: 0;
		font-family: var(--font-display);
		font-size: calc(var(--body-size) * 1.4);
		font-weight: var(--display-weight);
	}

	header p {
		margin: calc(var(--space) * 1.5) 0 0;
		color: var(--text-muted);
		font-family: var(--font-body);
		font-size: var(--body-size);
	}

	.pane {
		/* page.enter: fade, when the settings pane changes */
		animation: pane-enter var(--motion-page-enter);
	}

	footer {
		display: flex;
		justify-content: flex-end;
		gap: calc(var(--space) * 2);
		padding-top: calc(var(--space) * 1);
	}

	@keyframes dialog-enter {
		from {
			opacity: 0;
			transform: scale(0.9);
		}
	}

	@keyframes pane-enter {
		from {
			opacity: 0;
		}
	}
</style>
