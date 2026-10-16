import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { expectAccessible } from '../../test/axe';

vi.mock('$lib/api', async (importOriginal) => ({
	...(await importOriginal<typeof import('$lib/api')>()),
	listItems: vi.fn(),
	listKinds: vi.fn(),
	addItem: vi.fn(),
	onItemsChanged: vi.fn()
}));

import { CommandError, addItem, listItems, listKinds, onItemsChanged } from '$lib/api';
import type { ItemView, KindView } from '$lib/api';
import en from '$lib/i18n/en.json';
import Page from './+page.svelte';

const tools: KindView = {
	id: 'k-tools',
	label: { type: 'key', value: 'kind.tools' },
	counting: 'single',
	container: false
};
const furniture: KindView = {
	id: 'k-furniture',
	label: { type: 'key', value: 'kind.furniture' },
	counting: 'single',
	container: false
};
const item = (id: string, name: string, kind: KindView, quantity: number): ItemView => ({
	id,
	name,
	kind,
	quantity,
	created_ms: 0
});
const lamp = item('0000-aaaa', 'Floor lamp', furniture, 1);
const drill = item('0000-bbbb', 'Drill', tools, 2);

const mocks = {
	listItems: vi.mocked(listItems),
	listKinds: vi.mocked(listKinds),
	addItem: vi.mocked(addItem),
	onItemsChanged: vi.mocked(onItemsChanged)
};

beforeEach(() => {
	for (const mock of Object.values(mocks)) mock.mockReset();
	mocks.listKinds.mockResolvedValue([tools, furniture]);
	mocks.onItemsChanged.mockResolvedValue(() => {});
});

const rowNames = () =>
	screen
		.getAllByRole('row')
		.slice(1)
		.map((row) => within(row).getAllByRole('cell')[0].textContent);

describe('the Items screen', () => {
	it('lists the items with their kind and quantity', async () => {
		mocks.listItems.mockResolvedValue([lamp, drill]);

		render(Page);

		expect(await screen.findByRole('region', { name: en['items.title'] })).toBeInTheDocument();
		expect(rowNames()).toEqual([expect.stringContaining('Floor lamp'), expect.stringContaining('Drill')]);
		const cells = within(screen.getAllByRole('row')[2]).getAllByRole('cell');
		expect(cells.map((cell) => cell.textContent?.trim())).toEqual([
			expect.stringContaining('Drill'),
			en['kind.tools'],
			'2'
		]);
	});

	it('shows the empty state with a way to add the first item', async () => {
		mocks.listItems.mockResolvedValue([]);

		render(Page);

		expect(await screen.findByRole('heading', { name: en['items.empty.title'] })).toBeInTheDocument();
		expect(screen.getByRole('button', { name: en['items.add'] })).toBeInTheDocument();
	});

	it('shows an error with a retry when the items cannot be loaded, and recovers', async () => {
		const user = userEvent.setup();
		mocks.listItems.mockRejectedValueOnce(new CommandError('storage', 'disk full'));
		mocks.listItems.mockResolvedValue([lamp]);

		render(Page);

		expect(await screen.findByRole('heading', { name: en['items.error.title'] })).toBeInTheDocument();
		expect(screen.getByText(en['error.storage'])).toBeInTheDocument();
		await user.click(screen.getByRole('button', { name: en['items.error.retry'] }));
		expect(await screen.findByText('Floor lamp')).toBeInTheDocument();
	});

	it('loads again when the shell says items changed', async () => {
		mocks.listItems.mockResolvedValue([lamp]);
		render(Page);
		await screen.findByText('Floor lamp');
		const handler = mocks.onItemsChanged.mock.calls[0][0];

		mocks.listItems.mockResolvedValue([lamp, drill]);
		handler({ ids: [lamp.id] });

		expect(await screen.findByText('Drill')).toBeInTheDocument();
	});

	it('stops listening for changes when the screen goes away', async () => {
		const unlisten = vi.fn();
		mocks.listItems.mockResolvedValue([lamp]);
		mocks.onItemsChanged.mockResolvedValue(unlisten);

		const { unmount } = render(Page);
		await screen.findByText('Floor lamp');
		unmount();

		await waitFor(() => expect(unlisten).toHaveBeenCalledOnce());
	});

	it('sorts by a column when its header is clicked, and back the other way on the next click', async () => {
		const user = userEvent.setup();
		mocks.listItems.mockResolvedValue([lamp, drill]);
		render(Page);
		await screen.findByText('Floor lamp');

		await user.click(screen.getByRole('button', { name: en['items.column.name'] }));
		expect(rowNames()).toEqual([expect.stringContaining('Drill'), expect.stringContaining('Floor lamp')]);

		await user.click(screen.getByRole('button', { name: en['items.column.name'] }));
		expect(rowNames()).toEqual([expect.stringContaining('Floor lamp'), expect.stringContaining('Drill')]);

		await user.click(screen.getByRole('button', { name: en['items.column.quantity'] }));
		expect(rowNames()).toEqual([expect.stringContaining('Floor lamp'), expect.stringContaining('Drill')]);
		await user.click(screen.getByRole('button', { name: en['items.column.quantity'] }));
		expect(rowNames()).toEqual([expect.stringContaining('Drill'), expect.stringContaining('Floor lamp')]);
	});

	it('has no accessibility violations with items listed', async () => {
		mocks.listItems.mockResolvedValue([lamp, drill]);
		const { container } = render(Page);
		await screen.findByText('Floor lamp');

		await expectAccessible(container);
	});

	it('has no accessibility violations with the add form open', async () => {
		const user = userEvent.setup();
		mocks.listItems.mockResolvedValue([]);
		const { baseElement } = render(Page);
		await user.click(await screen.findByRole('button', { name: en['items.add'] }));
		await screen.findByRole('dialog', { name: en['item-form.title.new'] });

		await expectAccessible(baseElement);
	});
});

describe('adding an item', () => {
	async function openForm() {
		const user = userEvent.setup();
		mocks.listItems.mockResolvedValue([]);
		render(Page);
		await user.click(await screen.findByRole('button', { name: en['items.add'] }));
		const dialog = await screen.findByRole('dialog', { name: en['item-form.title.new'] });
		return { user, dialog };
	}

	const nameField = () => screen.getByRole('textbox', { name: en['item-form.name.label'] });
	const quantityField = () => screen.getByRole('textbox', { name: en['item-form.quantity.label'] });
	const save = () => screen.getByRole('button', { name: en['item-form.save'] });

	it('starts with an empty name, quantity 1 and the first kind', async () => {
		const { dialog } = await openForm();

		expect(nameField()).toHaveValue('');
		expect(quantityField()).toHaveValue('1');
		expect(within(dialog).getByText(en['kind.tools'])).toBeInTheDocument();
	});

	it('saves what was typed and closes the form', async () => {
		const { user, dialog } = await openForm();
		mocks.addItem.mockResolvedValue(drill);

		await user.type(nameField(), 'Drill');
		await user.clear(quantityField());
		await user.type(quantityField(), '2');
		await user.click(save());

		expect(mocks.addItem).toHaveBeenCalledExactlyOnceWith({ name: 'Drill', kind: 'k-tools', quantity: 2 });
		await waitFor(() => expect(dialog).not.toHaveAttribute('open'));
	});

	it('asks for a name without calling the shell', async () => {
		const { user, dialog } = await openForm();

		await user.click(save());

		expect(await within(dialog).findByText(en['item-form.name.required'])).toBeInTheDocument();
		expect(mocks.addItem).not.toHaveBeenCalled();
		expect(dialog).toHaveAttribute('open');
	});

	it.each(['0', '-1', '1.5', '', 'two', '1e3'])('refuses the quantity "%s" without calling the shell', async (bad) => {
		const { user, dialog } = await openForm();

		await user.type(nameField(), 'Drill');
		await user.clear(quantityField());
		if (bad) await user.type(quantityField(), bad);
		await user.click(save());

		expect(await within(dialog).findByText(en['item-form.quantity.invalid'])).toBeInTheDocument();
		expect(mocks.addItem).not.toHaveBeenCalled();
	});

	it('shows what the core refuses next to the field it is about', async () => {
		const { user, dialog } = await openForm();
		mocks.addItem.mockRejectedValue(new CommandError('quantity_invalid', 'too many'));

		await user.type(nameField(), 'Drill');
		await user.click(save());

		expect(await within(dialog).findByText(en['item-form.quantity.invalid'])).toBeInTheDocument();
		expect(dialog).toHaveAttribute('open');
	});

	it('shows any other failure as an alert and keeps what was typed', async () => {
		const { user } = await openForm();
		mocks.addItem.mockRejectedValue(new CommandError('storage', 'disk full'));

		await user.type(nameField(), 'Drill');
		await user.click(save());

		expect(await screen.findByRole('alert')).toHaveTextContent(en['error.storage']);
		expect(nameField()).toHaveValue('Drill');
	});

	it('forgets the last attempt when it is opened again', async () => {
		const { user } = await openForm();
		await user.type(nameField(), 'Drill');
		await user.click(screen.getByRole('button', { name: en['item-form.cancel'] }));

		await user.click(screen.getAllByRole('button', { name: en['items.add'] })[0]);

		expect(nameField()).toHaveValue('');
	});
});
