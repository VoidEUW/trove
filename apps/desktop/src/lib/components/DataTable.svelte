<script module lang="ts">
	import type { Snippet } from 'svelte';

	export type SortDirection = 'ascending' | 'descending';

	export interface Sort {
		/** The `id` of the sorted column. */
		column: string;
		direction: SortDirection;
	}

	export interface Column<Row> {
		id: string;
		/** The header text, already translated with `t()`. */
		header: string;
		/**
		 * `name` is the first column: an optional icon, the name and a short ID. `text` and `mono` show
		 * `value(row)`. A `cell` snippet replaces the content for anything richer (chips, avatars, a
		 * condition meter).
		 */
		type?: 'name' | 'text' | 'mono';
		value?: (row: Row) => string;
		/** The short ID shown beside the name in a `name` column. */
		detail?: (row: Row) => string;
		icon?: Snippet<[Row]>;
		cell?: Snippet<[Row]>;
		sortable?: boolean;
		align?: 'start' | 'end';
	}
</script>

<script lang="ts" generics="Row">
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';

	interface Props {
		/** The accessible name of the table, already translated with `t()`. */
		label: string;
		columns: Column<Row>[];
		rows: Row[];
		rowKey: (row: Row) => string;
		/** The current order. The caller sorts `rows`; the table only shows and reports it. */
		sort?: Sort | null;
		onsort?: (sort: Sort) => void;
		/** The key of the highlighted row. */
		selected?: string | null;
		/** Called when a row is clicked or opened with Enter or Space. */
		onopen?: (row: Row) => void;
		/** Trailing cell of each row, for an overflow menu. */
		actions?: Snippet<[Row]>;
		/** The accessible name of the actions column header, already translated with `t()`. */
		actionsLabel?: string;
		/** Shown instead of the table when there are no rows: a `ui/state-page`. */
		empty?: Snippet;
	}

	let {
		label,
		columns,
		rows,
		rowKey,
		sort = null,
		onsort,
		selected = $bindable(null),
		onopen,
		actions,
		actionsLabel = '',
		empty
	}: Props = $props();

	function ariaSort(column: Column<Row>): 'ascending' | 'descending' | 'none' | undefined {
		if (!column.sortable) return undefined;
		return sort?.column === column.id ? sort.direction : 'none';
	}

	function toggle(column: Column<Row>) {
		const direction: SortDirection =
			sort?.column === column.id && sort.direction === 'ascending' ? 'descending' : 'ascending';
		onsort?.({ column: column.id, direction });
	}

	function open(row: Row) {
		selected = rowKey(row);
		onopen?.(row);
	}

	function onrowkeydown(event: KeyboardEvent, row: Row) {
		if (event.target !== event.currentTarget) return;
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			open(row);
		}
	}
</script>

{#if rows.length === 0 && empty}
	{@render empty()}
{:else}
	<!-- A region that scrolls sideways has to be reachable by keyboard. -->
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<div class="data-table look-soft" role="region" aria-label={label} tabindex="0">
		<table>
			<thead>
				<tr>
					{#each columns as column (column.id)}
						<th
							scope="col"
							class:end={column.align === 'end'}
							class:sorted={sort?.column === column.id}
							aria-sort={ariaSort(column)}
						>
							{#if column.sortable}
								<button type="button" class="sort" onclick={() => toggle(column)}>
									{column.header}
									{#if sort?.column === column.id}
										<span class="direction" aria-hidden="true">
											{#if sort.direction === 'ascending'}<ChevronUp />{:else}<ChevronDown />{/if}
										</span>
									{/if}
								</button>
							{:else}
								{column.header}
							{/if}
						</th>
					{/each}
					{#if actions}
						<th scope="col" class="actions"><span class="visually-hidden">{actionsLabel}</span></th>
					{/if}
				</tr>
			</thead>
			<tbody>
				{#each rows as row (rowKey(row))}
					<!-- The row is also a mouse target; keyboard users get the same through tabindex and Enter. -->
					<!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_no_noninteractive_tabindex -->
					<tr
						tabindex="0"
						class:selected={selected === rowKey(row)}
						aria-current={selected === rowKey(row) ? 'true' : undefined}
						onclick={() => open(row)}
						onkeydown={(event) => onrowkeydown(event, row)}
					>
						{#each columns as column (column.id)}
							<td
								class:name={column.type === 'name'}
								class:mono={column.type === 'mono'}
								class:end={column.align === 'end'}
							>
								{#if column.cell}
									{@render column.cell(row)}
								{:else if column.type === 'name'}
									<span class="name-cell">
										{#if column.icon}<span class="icon">{@render column.icon(row)}</span>{/if}
										<span>{column.value?.(row)}</span>
										{#if column.detail}<small>{column.detail(row)}</small>{/if}
									</span>
								{:else}
									{column.value?.(row)}
								{/if}
							</td>
						{/each}
						{#if actions}
							<td class="actions">
								<!-- The menu button is its own click target, not a row click. -->
								<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
								<span onclick={(event) => event.stopPropagation()}>{@render actions(row)}</span>
							</td>
						{/if}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style>
	.data-table {
		--look-bg: var(--surface);
		overflow-x: auto;
		background: var(--look-bg);
		border: var(--border-width) solid var(--look-border);
		border-radius: var(--look-radius);
		box-shadow: var(--look-shadow);
		font-family: var(--font-ui);
		font-size: var(--ui-size);
		color: var(--text-muted);
	}

	table {
		width: 100%;
		border-collapse: collapse;
	}

	th {
		padding: calc(var(--space) * 2.5) calc(var(--space) * 3);
		border-bottom: var(--border-width) solid var(--line);
		font-size: calc(var(--ui-size) * 0.75);
		font-weight: 600;
		letter-spacing: 0.09em;
		text-align: start;
		text-transform: uppercase;
		white-space: nowrap;
		color: var(--text-faint);
	}

	th.sorted {
		color: var(--text);
	}

	th.end,
	td.end {
		text-align: end;
	}

	th.actions,
	td.actions {
		width: 1%;
		text-align: end;
	}

	.sort {
		display: inline-flex;
		align-items: center;
		gap: calc(var(--space) * 1);
		padding: 0;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		letter-spacing: inherit;
		text-transform: inherit;
		cursor: pointer;
	}

	.direction {
		display: inline-flex;
		color: var(--accent);
	}

	.direction :global(svg),
	.icon :global(svg) {
		width: var(--icon-size);
		height: var(--icon-size);
		stroke-width: var(--icon-stroke);
	}

	td {
		padding: calc(var(--space) * 2.5) calc(var(--space) * 3);
		border-bottom: var(--border-width) solid var(--line);
		transition: background-color var(--motion-entry-hover);
	}

	tbody tr:last-child td {
		border-bottom: 0;
	}

	tbody tr {
		cursor: pointer;
	}

	tbody tr:hover td {
		background: var(--hover);
	}

	tbody tr.selected td {
		background: var(--surface-raised);
	}

	tbody tr.selected td:first-child {
		box-shadow: inset 2px 0 0 var(--accent);
	}

	td.name {
		color: var(--text);
		font-family: var(--font-body);
		font-size: var(--body-size);
	}

	td.mono {
		font-family: var(--font-mono);
	}

	.name-cell {
		display: flex;
		align-items: center;
		gap: calc(var(--space) * 2);
		white-space: nowrap;
	}

	.icon {
		display: inline-flex;
		color: var(--text-muted);
	}

	small {
		font-family: var(--font-mono);
		font-size: calc(var(--ui-size) * 0.8);
		color: var(--text-faint);
	}

	tr:focus-visible,
	.sort:focus-visible,
	.data-table:focus-visible {
		outline: var(--focus-width) solid var(--focus);
		outline-offset: calc(var(--focus-width) * -1);
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip-path: inset(50%);
		white-space: nowrap;
	}
</style>
