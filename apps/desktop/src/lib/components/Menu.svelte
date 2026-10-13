<script module lang="ts">
	import type { Snippet } from 'svelte';

	export interface MenuItem {
		type?: 'item';
		id: string;
		label: string;
		icon?: Snippet;
		/** A shortcut hint, such as `⌘3`. */
		shortcut?: string;
		/** Set for a choice that can be on or off. A menu item with `radio` is one of a group. */
		checked?: boolean;
		radio?: boolean;
		danger?: boolean;
		disabled?: boolean;
		onselect?: () => void;
		/** Entries of a submenu that opens beside this item. */
		submenu?: MenuEntry[];
	}

	export interface MenuHeading {
		type: 'heading';
		label: string;
	}

	export interface MenuSeparator {
		type: 'separator';
	}

	export type MenuEntry = MenuItem | MenuHeading | MenuSeparator;
</script>

<script lang="ts">
	import Check from '@lucide/svelte/icons/check';
	import ChevronRight from '@lucide/svelte/icons/chevron-right';
	import Menu from './Menu.svelte';

	interface Props {
		/** The accessible name of the menu, already translated with `t()`. */
		label: string;
		entries: MenuEntry[];
		/** Called when the menu should close: an item was chosen, Escape, Tab or a click outside. */
		onclose: () => void;
		/** The element that opened the menu. A click on it is not an outside click. */
		anchor?: HTMLElement | null;
		/** Which edge of the parent the menu lines up with. */
		align?: 'start' | 'end';
		/** Set for a submenu. */
		nested?: boolean;
	}

	let { label, entries, onclose, anchor = null, align = 'start', nested = false }: Props = $props();

	let root = $state<HTMLDivElement>();
	let openSubmenu = $state<string | null>(null);

	function focusable(): HTMLElement[] {
		return Array.from(
			root?.querySelectorAll<HTMLElement>(':scope > [role^="menuitem"]:not(:disabled)') ?? []
		);
	}

	function focusAt(index: number) {
		const items = focusable();
		if (items.length) items[(index + items.length) % items.length].focus();
	}

	// A menu that opens puts focus on the checked item, or the first one.
	$effect(() => {
		if (!root) return;
		const items = focusable();
		(items.find((item) => item.getAttribute('aria-checked') === 'true') ?? items[0])?.focus();
	});

	// A click outside closes a menu, but not the one that opened it: its trigger toggles it.
	$effect(() => {
		if (nested) return;
		const close = (event: PointerEvent) => {
			const target = event.target as Node;
			if (!root?.contains(target) && !anchor?.contains(target)) onclose();
		};
		document.addEventListener('pointerdown', close);
		return () => document.removeEventListener('pointerdown', close);
	});

	function choose(item: MenuItem) {
		if (item.disabled) return;
		if (item.submenu) {
			openSubmenu = item.id;
			return;
		}
		item.onselect?.();
		onclose();
	}

	function onkeydown(event: KeyboardEvent) {
		const items = focusable();
		const index = items.indexOf(document.activeElement as HTMLElement);
		switch (event.key) {
			case 'ArrowDown':
				focusAt(index + 1);
				break;
			case 'ArrowUp':
				focusAt(index < 0 ? -1 : index - 1);
				break;
			case 'Home':
				focusAt(0);
				break;
			case 'End':
				focusAt(-1);
				break;
			case 'ArrowRight': {
				const item = entries.find((entry) => 'id' in entry && entry.id === items[index]?.dataset.id);
				if (item && 'submenu' in item && item.submenu) openSubmenu = item.id;
				else return;
				break;
			}
			case 'ArrowLeft':
				if (!nested) return;
				onclose();
				break;
			case 'Escape':
				event.stopPropagation();
				onclose();
				break;
			case 'Tab':
				onclose();
				return;
			default:
				return;
		}
		event.preventDefault();
	}

	function role(item: MenuItem): string {
		if (item.checked === undefined) return 'menuitem';
		return item.radio ? 'menuitemradio' : 'menuitemcheckbox';
	}
</script>

<!-- A menu handles the keys of the items inside it. -->
<!-- svelte-ignore a11y_interactive_supports_focus -->
<div
	bind:this={root}
	class="menu look-float"
	class:nested
	class:end={align === 'end'}
	role="menu"
	aria-label={label}
	{onkeydown}
>
	{#each entries as entry, index (index)}
		{#if entry.type === 'separator'}
			<hr />
		{:else if entry.type === 'heading'}
			<div class="heading" role="presentation">{entry.label}</div>
		{:else}
			<button
				type="button"
				class="item"
				class:danger={entry.danger}
				class:open={openSubmenu === entry.id}
				role={role(entry)}
				aria-checked={entry.checked}
				aria-haspopup={entry.submenu ? 'menu' : undefined}
				aria-expanded={entry.submenu ? openSubmenu === entry.id : undefined}
				data-id={entry.id}
				disabled={entry.disabled}
				tabindex="-1"
				onclick={() => choose(entry)}
				onpointerenter={() => (openSubmenu = entry.submenu ? entry.id : null)}
			>
				{#if entry.icon}<span class="icon" aria-hidden="true">{@render entry.icon()}</span>{/if}
				<span class="text">{entry.label}</span>
				{#if entry.checked}
					<span class="check icon" aria-hidden="true"><Check /></span>
				{:else if entry.shortcut}
					<kbd>{entry.shortcut}</kbd>
				{:else if entry.submenu}
					<span class="more icon" aria-hidden="true"><ChevronRight /></span>
				{/if}
			</button>
			{#if entry.submenu && openSubmenu === entry.id}
				<div class="submenu">
					<Menu
						label={entry.label}
						entries={entry.submenu}
						nested
						onclose={() => {
							openSubmenu = null;
							root?.querySelector<HTMLElement>(`[data-id="${entry.id}"]`)?.focus();
						}}
					/>
				</div>
			{/if}
		{/if}
	{/each}
</div>

<style>
	.menu {
		position: absolute;
		top: calc(100% + var(--space) * 1.5);
		left: 0;
		z-index: 10;
		min-width: 100%;
		width: max-content;
		padding: calc(var(--space) * 1.5);
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		background: var(--look-bg);
		box-shadow: var(--look-shadow);
		color: var(--text);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
		transform-origin: top left;
		/* menu.enter: scale */
		animation: menu-enter var(--motion-menu-enter);
	}

	.menu.end {
		left: auto;
		right: 0;
		transform-origin: top right;
	}

	.menu.nested {
		top: calc(var(--space) * -1.5);
		left: calc(100% + var(--space));
	}

	.submenu {
		position: relative;
	}

	.item {
		display: flex;
		align-items: center;
		gap: calc(var(--space) * 2);
		width: 100%;
		height: calc(var(--space) * 8);
		padding: 0 calc(var(--space) * 2.5);
		border: 0;
		border-radius: var(--radius-sm);
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: start;
		cursor: pointer;
	}

	.item:hover:not(:disabled),
	.item:focus-visible,
	.item.open {
		background: var(--hover);
	}

	.item:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(var(--focus-width) * -1);
	}

	.item:disabled {
		color: var(--text-faint);
		cursor: default;
	}

	.item.danger {
		color: var(--danger);
	}

	.text {
		flex: 1;
		white-space: nowrap;
	}

	.icon {
		display: inline-flex;
		color: var(--text-muted);
	}

	.check {
		color: var(--accent);
	}

	.icon :global(svg) {
		width: var(--icon-size);
		height: var(--icon-size);
		stroke-width: var(--icon-stroke);
	}

	kbd {
		font-family: var(--font-mono);
		font-size: calc(var(--ui-size) * 0.8);
		color: var(--text-faint);
	}

	.heading {
		padding: calc(var(--space) * 2) calc(var(--space) * 2.5) calc(var(--space) * 1);
		font-size: calc(var(--ui-size) * 0.75);
		font-weight: 600;
		letter-spacing: 0.09em;
		text-transform: uppercase;
		color: var(--text-faint);
	}

	hr {
		margin: calc(var(--space) * 1.5) 0;
		border: 0;
		border-top: var(--border-width) solid var(--line);
	}

	@keyframes menu-enter {
		from {
			opacity: 0;
			transform: scale(0.95);
		}
	}
</style>
