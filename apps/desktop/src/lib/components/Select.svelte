<script lang="ts" generics="Value extends string">
	import type { Snippet } from 'svelte';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import Menu from './Menu.svelte';

	interface Option {
		value: Value;
		/** The text of the choice, already translated with `t()`. */
		label: string;
	}

	interface Props {
		/** The visible label, already translated with `t()`. */
		label: string;
		options: Option[];
		/** The chosen value, or `null` while nothing is chosen. */
		value?: Value | null;
		placeholder?: string;
		help?: string;
		/** Why the choice is not accepted: shown in place of the help, in the error colour. */
		error?: string;
		disabled?: boolean;
		/** An icon before the value. */
		icon?: Snippet;
	}

	let {
		label,
		options,
		value = $bindable(null),
		placeholder = '',
		help,
		error,
		disabled = false,
		icon
	}: Props = $props();

	const id = $props.id();
	const noteId = `${id}-note`;

	let open = $state(false);
	let trigger = $state<HTMLButtonElement>();

	const current = $derived(options.find((option) => option.value === value));
	const entries = $derived(
		options.map((option) => ({
			id: option.value,
			label: option.label,
			checked: option.value === value,
			radio: true,
			onselect: () => (value = option.value)
		}))
	);

	function close() {
		open = false;
		trigger?.focus();
	}
</script>

<div class="select">
	<span class="label" {id}>{label}</span>
	<!-- The menu closes when focus leaves the select, for instance when the dialog around it closes. -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="anchor"
		onfocusout={(event) => {
			if (!event.currentTarget.contains(event.relatedTarget as Node | null)) open = false;
		}}
	>
		<button
			bind:this={trigger}
			type="button"
			class="trigger look-soft"
			{disabled}
			aria-labelledby="{id} {id}-value"
			aria-haspopup="menu"
			aria-expanded={open}
			class:invalid={!!error}
			aria-describedby={error || help ? noteId : undefined}
			onclick={() => (open = !open)}
			onkeydown={(event) => {
				if (event.key === 'ArrowDown' && !open) {
					event.preventDefault();
					open = true;
				}
			}}
		>
			{#if icon}<span class="icon" aria-hidden="true">{@render icon()}</span>{/if}
			<span id="{id}-value" class="value" class:placeholder={!current}>
				{current?.label ?? placeholder}
			</span>
			<span class="icon chevron" aria-hidden="true"><ChevronDown /></span>
		</button>
		{#if open}
			<Menu {label} {entries} anchor={trigger} onclose={close} />
		{/if}
	</div>
	{#if error || help}
		<span id={noteId} class="note" class:error>{error ?? help}</span>
	{/if}
</div>

<style>
	.select {
		display: grid;
		gap: calc(var(--space) * 1.5);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
	}

	.label {
		color: var(--text-muted);
	}

	.anchor {
		position: relative;
	}

	.trigger {
		display: flex;
		align-items: center;
		gap: calc(var(--space) * 2);
		width: 100%;
		min-height: calc(var(--space) * 9);
		padding: 0 calc(var(--space) * 3);
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		background: var(--look-bg);
		box-shadow:
			var(--look-shadow),
			inset 0 0 0 0 var(--focus);
		color: var(--text);
		font: inherit;
		text-align: start;
		cursor: pointer;
		/* field.focus: underline */
		transition: box-shadow var(--motion-field-focus);
	}

	.trigger:focus {
		box-shadow:
			var(--look-shadow),
			inset 0 calc(var(--focus-width) * -1) 0 0 var(--focus);
	}

	.trigger:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: var(--focus-width);
	}

	.trigger.invalid {
		border-color: var(--danger);
	}

	.trigger:disabled {
		color: var(--text-faint);
		cursor: default;
	}

	.value {
		flex: 1;
	}

	.placeholder {
		color: var(--text-faint);
	}

	.icon {
		display: inline-flex;
		color: var(--text-faint);
	}

	.icon :global(svg) {
		width: var(--icon-size);
		height: var(--icon-size);
		stroke-width: var(--icon-stroke);
	}

	.note {
		font-size: calc(var(--ui-size) * 0.86);
		color: var(--text-faint);
	}

	.note.error {
		color: var(--danger);
	}
</style>
