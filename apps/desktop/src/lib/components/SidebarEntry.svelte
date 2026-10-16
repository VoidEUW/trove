<script lang="ts">
	import { getContext, type Snippet } from 'svelte';
	import { sidebarContext, type SidebarContext } from './Sidebar.svelte';

	interface Props {
		/** The text of the entry, already translated with `t()`. */
		label: string;
		/** Makes the entry a link. Without it the entry is a button. */
		href?: string;
		icon?: Snippet;
		/** A small note at the end, such as a count. */
		meta?: string;
		/** An action at the end, such as a menu button. */
		trailing?: Snippet;
		active?: boolean;
		disabled?: boolean;
		onclick?: (event: MouseEvent) => void;
	}

	let { label, href, icon, meta, trailing, active = false, disabled = false, onclick }: Props =
		$props();

	const sidebar = getContext<SidebarContext | undefined>(sidebarContext);
	const collapsed = $derived(sidebar?.collapsed ?? false);
	const link = $derived(href !== undefined && !disabled);
</script>

{#snippet content()}
	{#if icon}<span class="icon" aria-hidden="true">{@render icon()}</span>{/if}
	<span class="label">{label}</span>
	{#if meta}<span class="meta">{meta}</span>{/if}
{/snippet}

<div class="entry look-soft" class:active class:collapsed class:disabled>
	{#if link}
		<a
			class="row"
			{href}
			aria-current={active ? 'page' : undefined}
			title={collapsed ? label : undefined}
			{onclick}
		>
			{@render content()}
		</a>
	{:else}
		<button
			type="button"
			class="row"
			{disabled}
			aria-current={active ? 'page' : undefined}
			title={collapsed ? label : undefined}
			{onclick}
		>
			{@render content()}
		</button>
	{/if}
	{#if trailing && !collapsed}<span class="trailing">{@render trailing()}</span>{/if}
</div>

<style>
	.entry {
		display: flex;
		align-items: center;
		border-radius: var(--radius-sm);
		color: var(--text-muted);
		/* entry.hover: tint */
		transition: background-color var(--motion-entry-hover);
	}

	.entry:hover:not(.disabled) {
		background: var(--hover);
		color: var(--text);
	}

	.row {
		display: flex;
		flex: 1;
		align-items: center;
		gap: calc(var(--space) * 2);
		min-width: 0;
		height: calc(var(--space) * 8);
		padding: 0 calc(var(--space) * 2);
		border: 0;
		border-radius: inherit;
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: start;
		text-decoration: none;
		cursor: pointer;
	}

	.row:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(var(--focus-width) * -1);
	}

	.entry {
		position: relative;
	}

	/* entry.active: bar */
	.entry::before {
		content: '';
		position: absolute;
		inset-block: 0;
		inset-inline-start: 0;
		width: 2px;
		background: var(--accent);
		opacity: 0;
		transition: opacity var(--motion-entry-active);
	}

	.active {
		color: var(--text);
	}

	.active::before {
		opacity: 1;
	}

	.disabled {
		color: var(--text-faint);
	}

	.disabled .row {
		cursor: default;
	}

	.icon {
		display: inline-flex;
		flex: none;
	}

	.icon :global(svg) {
		width: var(--icon-size);
		height: var(--icon-size);
		stroke-width: var(--icon-stroke);
	}

	.label {
		flex: 1;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
		transition: opacity var(--motion-sidebar-collapse);
	}

	.collapsed .label,
	.collapsed .meta {
		opacity: 0;
		width: 0;
		flex: none;
	}

	.meta {
		font-size: calc(var(--ui-size) * 0.8);
		color: var(--text-faint);
	}

	.trailing {
		display: inline-flex;
		padding-inline-end: calc(var(--space) * 1);
	}
</style>
