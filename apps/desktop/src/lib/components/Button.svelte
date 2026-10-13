<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import LoaderCircle from '@lucide/svelte/icons/loader-circle';

	interface Props extends Omit<HTMLButtonAttributes, 'children'> {
		/** `primary` is the one action a screen leads with. */
		variant?: 'neutral' | 'primary' | 'danger' | 'ghost';
		size?: 'sm' | 'md' | 'lg';
		/** A leading icon. */
		icon?: Snippet;
		/** The accessible name of an icon-only button, already translated with `t()`. */
		label?: string;
		/** Shows a spinner instead of the icon and ignores clicks. */
		loading?: boolean;
		children?: Snippet;
	}

	let {
		variant = 'neutral',
		size = 'md',
		icon,
		label,
		loading = false,
		disabled = false,
		type = 'button',
		children,
		onclick,
		class: className = '',
		...rest
	}: Props = $props();

	function click(event: MouseEvent & { currentTarget: EventTarget & HTMLButtonElement }) {
		if (loading) {
			event.preventDefault();
			return;
		}
		onclick?.(event);
	}
</script>

<button
	{...rest}
	{type}
	{disabled}
	class="button look-soft {variant} {size} {className}"
	class:icon-only={!children}
	aria-label={children ? undefined : label}
	aria-busy={loading || undefined}
	onclick={click}
>
	{#if loading}
		<span class="icon spinner" aria-hidden="true"><LoaderCircle /></span>
	{:else if icon}
		<span class="icon" aria-hidden="true">{@render icon()}</span>
	{/if}
	{@render children?.()}
</button>

<style>
	.button {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: calc(var(--space) * 1.5);
		height: calc(var(--space) * 8);
		padding: 0 calc(var(--space) * 3.5);
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		background: var(--look-bg);
		box-shadow: var(--look-shadow);
		color: var(--text);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
		font-weight: 500;
		white-space: nowrap;
		cursor: pointer;
		transition:
			transform var(--motion-button-hover),
			box-shadow var(--motion-button-hover),
			background-color var(--motion-button-hover);
	}

	.sm {
		height: calc(var(--space) * 7);
		padding: 0 calc(var(--space) * 2.5);
	}

	.lg {
		height: calc(var(--space) * 10);
		padding: 0 calc(var(--space) * 5);
	}

	.icon-only {
		width: calc(var(--space) * 8);
		padding: 0;
	}

	.icon-only.sm {
		width: calc(var(--space) * 7);
	}

	.icon-only.lg {
		width: calc(var(--space) * 10);
	}

	.primary {
		--look-bg: var(--accent);
		color: var(--on-accent);
	}

	.danger {
		--look-bg: var(--danger);
		color: var(--on-danger);
	}

	.ghost {
		--look-bg: transparent;
		color: var(--text-muted);
	}

	.ghost:hover:not(:disabled) {
		background: var(--hover);
		color: var(--text);
	}

	/* button.hover: lift */
	.button:hover:not(:disabled):not([aria-busy]) {
		transform: translateY(-1px);
		box-shadow: var(--shadow-sm);
	}

	/* button.press: sink */
	.button:active:not(:disabled):not([aria-busy]) {
		transform: translateY(1px);
		box-shadow: none;
		transition:
			transform var(--motion-button-press),
			box-shadow var(--motion-button-press);
	}

	.button:disabled {
		--look-bg: var(--surface-raised);
		color: var(--text-faint);
		cursor: default;
	}

	.ghost:disabled {
		--look-bg: transparent;
	}

	.button:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: var(--focus-width);
	}

	.icon {
		display: inline-flex;
	}

	.icon :global(svg) {
		width: var(--icon-size);
		height: var(--icon-size);
		stroke-width: var(--icon-stroke);
	}

	.spinner {
		animation: spin var(--motion-spinner-loop) infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(1turn);
		}
	}
</style>
