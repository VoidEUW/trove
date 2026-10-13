<script module lang="ts">
	/** Lets the entries inside a sidebar know whether it is collapsed to the icon rail. */
	export const sidebarContext = Symbol('sidebar');

	export interface SidebarContext {
		readonly collapsed: boolean;
	}

	/** The width limits, in multiples of `--space`. */
	export const MIN_WIDTH = 44;
	export const MAX_WIDTH = 80;
</script>

<script lang="ts">
	import { setContext, type Snippet } from 'svelte';
	import PanelLeftClose from '@lucide/svelte/icons/panel-left-close';
	import PanelLeftOpen from '@lucide/svelte/icons/panel-left-open';
	import Button from './Button.svelte';

	interface Props {
		/** The accessible name of the navigation, already translated with `t()`. */
		label: string;
		/** The name shown in the head. */
		title: string;
		/** The mark before the title. */
		mark?: Snippet;
		collapsed?: boolean;
		/** The width in multiples of `--space`, from `MIN_WIDTH` to `MAX_WIDTH`. */
		width?: number;
		/** Accessible names of the collapse button and the resize edge, already translated. */
		collapseLabel: string;
		expandLabel: string;
		resizeLabel: string;
		/** The body: groups and entries. */
		children?: Snippet;
		/** The foot: one entry. */
		foot?: Snippet;
	}

	let {
		label,
		title,
		mark,
		collapsed = $bindable(false),
		width = $bindable(52),
		collapseLabel,
		expandLabel,
		resizeLabel,
		children,
		foot
	}: Props = $props();

	setContext<SidebarContext>(sidebarContext, {
		get collapsed() {
			return collapsed;
		}
	});

	let sidebar = $state<HTMLElement>();
	let dragging = $state(false);

	function clamp(value: number): number {
		return Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, Math.round(value)));
	}

	function drag(event: PointerEvent) {
		if (!dragging || !sidebar) return;
		const unit = parseFloat(getComputedStyle(sidebar).getPropertyValue('--space'));
		width = clamp((event.clientX - sidebar.getBoundingClientRect().left) / unit);
	}

	function resizeKey(event: KeyboardEvent) {
		const step = event.key === 'ArrowRight' ? 2 : event.key === 'ArrowLeft' ? -2 : 0;
		if (!step) return;
		event.preventDefault();
		width = clamp(width + step);
	}
</script>

<aside
	bind:this={sidebar}
	class="sidebar look-float"
	class:collapsed
	class:dragging
	style:--sidebar-width="calc(var(--space) * {width})"
>
	<header>
		{#if mark}<span class="mark" aria-hidden="true">{@render mark()}</span>{/if}
		<span class="title">{title}</span>
		<Button
			variant="ghost"
			size="sm"
			label={collapsed ? expandLabel : collapseLabel}
			aria-expanded={!collapsed}
			onclick={() => (collapsed = !collapsed)}
		>
			{#snippet icon()}
				{#if collapsed}<PanelLeftOpen />{:else}<PanelLeftClose />{/if}
			{/snippet}
		</Button>
	</header>
	<nav aria-label={label}>{@render children?.()}</nav>
	{#if foot}<footer>{@render foot()}</footer>{/if}
	{#if !collapsed}
		<!-- A separator you can focus is a slider for the width: arrow keys resize it. -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
		<div
			class="edge"
			role="separator"
			aria-orientation="vertical"
			aria-label={resizeLabel}
			aria-valuenow={width}
			aria-valuemin={MIN_WIDTH}
			aria-valuemax={MAX_WIDTH}
			tabindex="0"
			onpointerdown={(event) => {
				dragging = true;
				event.currentTarget.setPointerCapture(event.pointerId);
			}}
			onpointermove={drag}
			onpointerup={() => (dragging = false)}
			onpointercancel={() => (dragging = false)}
			onkeydown={resizeKey}
		></div>
	{/if}
</aside>

<style>
	.sidebar {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: calc(var(--space) * 2);
		width: var(--sidebar-width);
		flex: none;
		padding: calc(var(--space) * 3);
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		background: var(--look-bg);
		box-shadow: var(--look-shadow);
		color: var(--text);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
		overflow: hidden;
		/* sidebar.collapse: width */
		transition: width var(--motion-sidebar-collapse);
	}

	.sidebar.dragging {
		transition: none;
		user-select: none;
	}

	.sidebar.collapsed {
		width: calc(var(--space) * 16);
	}

	header {
		display: flex;
		align-items: center;
		gap: calc(var(--space) * 2);
		min-height: calc(var(--space) * 8);
		padding: 0 calc(var(--space) * 1);
	}

	.mark {
		display: inline-flex;
		color: var(--accent);
	}

	.mark :global(svg) {
		width: var(--icon-size);
		height: var(--icon-size);
		stroke-width: var(--icon-stroke);
	}

	.title {
		flex: 1;
		font-family: var(--font-display);
		font-size: calc(var(--body-size) * 1.25);
		font-weight: var(--display-weight);
		white-space: nowrap;
		transition: opacity var(--motion-sidebar-collapse);
	}

	.collapsed .title {
		opacity: 0;
		pointer-events: none;
	}

	.collapsed header {
		flex-direction: column;
		align-items: flex-start;
	}

	nav {
		display: grid;
		align-content: start;
		gap: calc(var(--space) * 1);
		flex: 1;
		overflow-y: auto;
	}

	footer {
		display: grid;
		gap: calc(var(--space) * 1);
	}

	.edge {
		position: absolute;
		top: 0;
		right: 0;
		bottom: 0;
		width: calc(var(--space) * 1.5);
		cursor: col-resize;
	}

	.edge:hover,
	.dragging .edge {
		background: var(--line);
	}

	.edge:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(var(--focus-width) * -1);
	}
</style>
