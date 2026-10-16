import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import DataTable, { type Column, type Sort } from './DataTable.svelte';

interface Row {
	id: string;
	name: string;
	kind: string;
	quantity: number;
}

const rows: Row[] = [
	{ id: 'a1', name: 'Floor lamp', kind: 'Electrical', quantity: 1 },
	{ id: 'b2', name: 'Screws', kind: 'Parts', quantity: 40 }
];

const columns: Column<Row>[] = [
	{
		id: 'name',
		header: 'Name',
		type: 'name',
		value: (row) => row.name,
		detail: (row) => row.id.toUpperCase(),
		sortable: true
	},
	{ id: 'kind', header: 'Kind', value: (row) => row.kind, sortable: true },
	{
		id: 'quantity',
		header: 'Quantity',
		type: 'mono',
		value: (row) => String(row.quantity),
		align: 'end'
	}
];

function setup(props: Partial<Parameters<typeof render<typeof DataTable<Row>>>[1]> = {}) {
	return render(DataTable<Row>, {
		label: 'Items',
		columns,
		rows,
		rowKey: (row: Row) => row.id,
		...props
	});
}

const dataRows = () => screen.getAllByRole('row').slice(1);
const header = (name: string) => screen.getByRole('columnheader', { name });

describe('ui/data-table', () => {
	it('shows a labelled region with one header per column and one row per item', () => {
		setup();

		expect(screen.getByRole('region', { name: 'Items' })).toHaveClass('look-soft');
		expect(screen.getAllByRole('columnheader').map((th) => th.textContent?.trim())).toEqual([
			'Name',
			'Kind',
			'Quantity'
		]);
		expect(dataRows()).toHaveLength(2);
		expect(within(dataRows()[1]).getByText('Screws')).toBeInTheDocument();
		expect(within(dataRows()[1]).getByText('40')).toBeInTheDocument();
	});

	it('shows the short id beside the name, and numbers in the mono style', () => {
		setup();

		expect(within(dataRows()[0]).getByText('A1')).toBeInTheDocument();
		expect(screen.getByText('40').closest('td')).toHaveClass('mono', 'end');
		expect(screen.getByText('Floor lamp').closest('td')).toHaveClass('name');
	});

	describe('sorting', () => {
		it('marks only sortable columns with aria-sort, and "none" until one is sorted', () => {
			setup();

			expect(header('Name')).toHaveAttribute('aria-sort', 'none');
			expect(header('Kind')).toHaveAttribute('aria-sort', 'none');
			expect(header('Quantity')).not.toHaveAttribute('aria-sort');
		});

		it('reports the direction of the sorted column and none for the others', () => {
			setup({ sort: { column: 'kind', direction: 'descending' } });

			expect(header('Kind')).toHaveAttribute('aria-sort', 'descending');
			expect(header('Name')).toHaveAttribute('aria-sort', 'none');
		});

		it('asks for ascending on the first click, descending on the next, and ascending on another column', async () => {
			const onsort = vi.fn<(sort: Sort) => void>();
			const user = userEvent.setup();
			const view = setup({ onsort });

			await user.click(screen.getByRole('button', { name: 'Name' }));
			expect(onsort).toHaveBeenLastCalledWith({ column: 'name', direction: 'ascending' });

			await view.rerender({ sort: { column: 'name', direction: 'ascending' } });
			await user.click(screen.getByRole('button', { name: 'Name' }));
			expect(onsort).toHaveBeenLastCalledWith({ column: 'name', direction: 'descending' });

			await view.rerender({ sort: { column: 'name', direction: 'descending' } });
			await user.click(screen.getByRole('button', { name: 'Name' }));
			expect(onsort).toHaveBeenLastCalledWith({ column: 'name', direction: 'ascending' });

			await user.click(screen.getByRole('button', { name: 'Kind' }));
			expect(onsort).toHaveBeenLastCalledWith({ column: 'kind', direction: 'ascending' });
		});

		it('does not reorder the rows itself: the caller sorts', async () => {
			const user = userEvent.setup();
			setup({ onsort: () => {} });

			await user.click(screen.getByRole('button', { name: 'Name' }));

			expect(dataRows().map((row) => row.textContent)).toEqual([
				expect.stringContaining('Floor lamp'),
				expect.stringContaining('Screws')
			]);
		});

		it('has no sort button on a column that is not sortable', () => {
			setup();

			expect(screen.queryByRole('button', { name: 'Quantity' })).not.toBeInTheDocument();
		});
	});

	describe('selecting a row', () => {
		it('selects and opens a row on click', async () => {
			const onopen = vi.fn();
			const user = userEvent.setup();
			setup({ onopen });

			await user.click(screen.getByText('Screws'));

			expect(onopen).toHaveBeenCalledWith(rows[1]);
			expect(dataRows()[1]).toHaveAttribute('aria-current', 'true');
			expect(dataRows()[0]).not.toHaveAttribute('aria-current');
			expect(dataRows()[1]).toHaveClass('selected');
		});

		it.each([
			['Enter', '{Enter}'],
			['Space', ' ']
		])('opens the focused row with %s', async (_, keys) => {
			const onopen = vi.fn();
			const user = userEvent.setup();
			setup({ onopen });

			dataRows()[0].focus();
			await user.keyboard(keys);

			expect(onopen).toHaveBeenCalledExactlyOnceWith(rows[0]);
			expect(dataRows()[0]).toHaveAttribute('aria-current', 'true');
		});

		it('ignores other keys', async () => {
			const onopen = vi.fn();
			const user = userEvent.setup();
			setup({ onopen });

			dataRows()[0].focus();
			await user.keyboard('a{ArrowDown}{Escape}');

			expect(onopen).not.toHaveBeenCalled();
		});

		it('shows the row named by selected before anything is clicked', () => {
			setup({ selected: 'a1' });

			expect(dataRows()[0]).toHaveAttribute('aria-current', 'true');
			expect(dataRows()[1]).not.toHaveAttribute('aria-current');
		});

		it('lets every row be reached with the keyboard', () => {
			setup();

			expect(dataRows().every((row) => row.getAttribute('tabindex') === '0')).toBe(true);
			expect(screen.getByRole('region')).toHaveAttribute('tabindex', '0');
		});
	});

	describe('actions', () => {
		const actions = createRawSnippet<[Row]>((row) => ({
			render: () => `<button type="button">More ${row().name}</button>`
		}));

		it('adds a trailing cell per row, with a header that has an accessible name', () => {
			setup({ actions, actionsLabel: 'Actions' });

			expect(header('Actions')).toBeInTheDocument();
			expect(screen.getByRole('button', { name: 'More Screws' })).toBeInTheDocument();
		});

		it('does not select the row when the actions cell is clicked', async () => {
			const onopen = vi.fn();
			const user = userEvent.setup();
			setup({ actions, actionsLabel: 'Actions', onopen });

			await user.click(screen.getByRole('button', { name: 'More Screws' }));

			expect(onopen).not.toHaveBeenCalled();
			expect(dataRows()[1]).not.toHaveAttribute('aria-current');
		});

		it('does not open the row when Enter is pressed on a button inside it', async () => {
			const onopen = vi.fn();
			const user = userEvent.setup();
			setup({ actions, actionsLabel: 'Actions', onopen });

			screen.getByRole('button', { name: 'More Screws' }).focus();
			await user.keyboard('{Enter}');

			expect(onopen).not.toHaveBeenCalled();
		});
	});

	describe('without rows', () => {
		const empty = createRawSnippet(() => ({ render: () => '<p>No items yet</p>' }));

		it('shows the empty snippet instead of the table', () => {
			setup({ rows: [], empty });

			expect(screen.getByText('No items yet')).toBeInTheDocument();
			expect(screen.queryByRole('table')).not.toBeInTheDocument();
		});

		it('shows only the header when there is no empty snippet', () => {
			setup({ rows: [] });

			expect(screen.getByRole('table')).toBeInTheDocument();
			expect(dataRows()).toHaveLength(0);
		});

		it('shows the rows, not the empty snippet, once there are some', () => {
			setup({ empty });

			expect(screen.queryByText('No items yet')).not.toBeInTheDocument();
			expect(dataRows()).toHaveLength(2);
		});
	});

	it('renders a cell snippet in place of the value', () => {
		const cell = createRawSnippet<[Row]>((row) => ({
			render: () => `<strong>${row().kind.toUpperCase()}</strong>`
		}));
		setup({ columns: [{ id: 'kind', header: 'Kind', cell }] });

		expect(screen.getByText('PARTS').tagName).toBe('STRONG');
	});
});
