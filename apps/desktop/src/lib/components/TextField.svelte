<script lang="ts">
	interface Props {
		/** The visible label, already translated with `t()`. */
		label: string;
		value?: string;
		placeholder?: string;
		/** A hint under the field. An `error` replaces it. */
		help?: string;
		/** Why the value is not accepted: shown in place of the help, in the error colour. */
		error?: string;
		disabled?: boolean;
		required?: boolean;
		name?: string;
		type?: 'text' | 'number' | 'search' | 'email' | 'url' | 'password';
		inputmode?: 'text' | 'numeric' | 'decimal' | 'search' | 'email' | 'url';
		autocomplete?: 'off' | 'on';
		/** A multi-line field. */
		multiline?: boolean;
		rows?: number;
		/** The most characters allowed. With `counter`, the count is shown. */
		maxlength?: number;
		counter?: boolean;
		oninput?: (event: Event) => void;
	}

	let {
		label,
		value = $bindable(''),
		placeholder,
		help,
		error,
		disabled = false,
		required = false,
		name,
		type = 'text',
		inputmode,
		autocomplete,
		multiline = false,
		rows = 3,
		maxlength,
		counter = false,
		oninput
	}: Props = $props();

	const id = $props.id();
	const noteId = `${id}-note`;
</script>

<div class="text-field">
	<label for={id}>{label}</label>
	{#if multiline}
		<textarea
			{id}
			{name}
			{rows}
			{placeholder}
			{disabled}
			{required}
			{maxlength}
			class="control look-open"
			bind:value
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={error || help ? noteId : undefined}
			{oninput}
		></textarea>
	{:else}
		<input
			{id}
			{name}
			{type}
			{inputmode}
			{autocomplete}
			{placeholder}
			{disabled}
			{required}
			{maxlength}
			class="control look-open"
			bind:value
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={error || help ? noteId : undefined}
			{oninput}
		/>
	{/if}
	{#if error || help || (counter && maxlength)}
		<div class="note">
			<span id={noteId} class:error>{error ?? help ?? ''}</span>
			{#if counter && maxlength}
				<span class="counter">{value.length} / {maxlength}</span>
			{/if}
		</div>
	{/if}
</div>

<style>
	.text-field {
		display: grid;
		gap: calc(var(--space) * 1.5);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
	}

	label {
		color: var(--text-muted);
	}

	.control {
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
		/* field.focus: underline */
		transition: box-shadow var(--motion-field-focus);
	}

	textarea.control {
		padding: calc(var(--space) * 2) calc(var(--space) * 3);
		line-height: 1.5;
		resize: vertical;
	}

	.control::placeholder {
		color: var(--text-faint);
	}

	.control:focus {
		box-shadow:
			var(--look-shadow),
			inset 0 calc(var(--focus-width) * -1) 0 0 var(--focus);
	}

	.control:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: var(--focus-width);
	}

	.control[aria-invalid='true'] {
		border-color: var(--danger);
	}

	.control:disabled {
		background: var(--surface-raised);
		color: var(--text-faint);
		cursor: default;
	}

	.note {
		display: flex;
		justify-content: space-between;
		gap: calc(var(--space) * 3);
		font-size: calc(var(--ui-size) * 0.86);
		color: var(--text-faint);
	}

	.note .error {
		color: var(--danger);
	}

	.counter {
		margin-inline-start: auto;
		font-family: var(--font-mono);
	}
</style>
