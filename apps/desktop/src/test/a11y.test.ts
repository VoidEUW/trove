import { render } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Button from '$lib/components/Button.svelte';
import DataTable from '$lib/components/DataTable.svelte';
import Dialog from '$lib/components/Dialog.svelte';
import Menu from '$lib/components/Menu.svelte';
import Select from '$lib/components/Select.svelte';
import Sidebar from '$lib/components/Sidebar.svelte';
import SidebarEntry from '$lib/components/SidebarEntry.svelte';
import StatePage from '$lib/components/StatePage.svelte';
import TextField from '$lib/components/TextField.svelte';
import Topbar from '$lib/components/Topbar.svelte';
import { expectAccessible } from './axe';

const text = (content: string) => createRawSnippet(() => ({ render: () => `<span>${content}</span>` }));

describe('accessibility of the components (axe)', () => {
	it.each(['neutral', 'primary', 'danger', 'ghost'] as const)('button, %s', async (variant) => {
		const { container } = render(Button, { variant, children: text('Save') });
		await expectAccessible(container);
	});

	it('button, icon only with a label, loading and disabled', async () => {
		const icon = createRawSnippet(() => ({ render: () => '<svg aria-hidden="true"></svg>' }));
		const { container } = render(Button, { icon, label: 'Close' });
		const loading = render(Button, { loading: true, children: text('Saving') });
		const disabled = render(Button, { disabled: true, children: text('Save') });
		await expectAccessible(container);
		await expectAccessible(loading.container);
		await expectAccessible(disabled.container);
	});

	it('text field, with help, with an error, required and multi-line', async () => {
		const plain = render(TextField, { label: 'Name', help: 'What it is called' });
		const failed = render(TextField, { label: 'Name', error: 'A name is required', required: true });
		const long = render(TextField, { label: 'Notes', multiline: true, maxlength: 80, counter: true });
		await expectAccessible(plain.container);
		await expectAccessible(failed.container);
		await expectAccessible(long.container);
	});

	it('select, closed, with an error and disabled', async () => {
		const options = [
			{ value: 'a', label: 'Tools' },
			{ value: 'b', label: 'Furniture' }
		];
		const plain = render(Select, { label: 'Kind', options, value: 'a' });
		const failed = render(Select, { label: 'Kind', options, error: 'Choose a kind' });
		const disabled = render(Select, { label: 'Kind', options, disabled: true });
		await expectAccessible(plain.container);
		await expectAccessible(failed.container);
		await expectAccessible(disabled.container);
	});

	it('menu, with items, a heading, a separator, a choice and a danger item', async () => {
		const { container } = render(Menu, {
			label: 'Item actions',
			onclose: () => {},
			entries: [
				{ type: 'heading', label: 'Item' },
				{ id: 'open', label: 'Open', shortcut: '⌘O' },
				{ id: 'archived', label: 'Archived', checked: true },
				{ type: 'separator' },
				{ id: 'delete', label: 'Delete', danger: true }
			]
		});
		await expectAccessible(container);
	});

	it('dialog, open, with a description and actions', async () => {
		const { baseElement } = render(Dialog, {
			open: true,
			title: 'New item',
			description: 'Add something you own',
			children: text('Body'),
			actions: createRawSnippet(() => ({ render: () => '<button type="button">Cancel</button>' }))
		});
		await expectAccessible(baseElement);
	});

	it.each(['empty', 'error', 'offline', 'not-found'] as const)('state page, %s', async (kind) => {
		const { container } = render(StatePage, {
			kind,
			title: 'Nothing here',
			description: 'Add the first one',
			primary: { label: 'Add', onclick: () => {} },
			secondary: { label: 'Back', onclick: () => {} },
			details: 'storage: disk full',
			detailsLabel: 'Details'
		});
		await expectAccessible(container);
	});

	it('data table, with sorting, a selected row and an actions column', async () => {
		const { container } = render(DataTable<{ id: string; name: string }>, {
			label: 'Items',
			columns: [{ id: 'name', header: 'Name', type: 'name', value: (r) => r.name, sortable: true }],
			rows: [
				{ id: 'a', name: 'Lamp' },
				{ id: 'b', name: 'Chair' }
			],
			rowKey: (r) => r.id,
			sort: { column: 'name', direction: 'ascending' },
			selected: 'a',
			actionsLabel: 'Actions',
			actions: createRawSnippet(() => ({ render: () => '<button type="button">More</button>' }))
		});
		await expectAccessible(container);
	});

	it('sidebar, expanded and collapsed, with entries', async () => {
		const children = createRawSnippet(() => ({
			render: () => '<a href="/items">Items</a>'
		}));
		const props = {
			label: 'Main',
			title: 'Trove',
			collapseLabel: 'Collapse',
			expandLabel: 'Expand',
			resizeLabel: 'Resize',
			children
		};
		const open = render(Sidebar, props);
		const collapsed = render(Sidebar, { ...props, collapsed: true });
		await expectAccessible(open.container);
		await expectAccessible(collapsed.container);
	});

	it('sidebar entry, as a link, a button, active and disabled', async () => {
		const link = render(SidebarEntry, { label: 'Items', href: '/items', active: true });
		const button = render(SidebarEntry, { label: 'Settings' });
		const disabled = render(SidebarEntry, { label: 'Soon', disabled: true });
		await expectAccessible(link.container);
		await expectAccessible(button.container);
		await expectAccessible(disabled.container);
	});

	it('top bar, with a toolbar', async () => {
		const toolbar = createRawSnippet(() => ({ render: () => '<button type="button">Add</button>' }));
		const { container } = render(Topbar, { title: 'Items', toolbar });
		await expectAccessible(container);
	});
});

describe('the accessibility check itself', () => {
	it('fails on markup with a known problem, so a pass means something', async () => {
		const { container } = render(Button, {
			icon: createRawSnippet(() => ({ render: () => '<svg aria-hidden="true"></svg>' }))
		});
		await expect(expectAccessible(container)).rejects.toThrow(/button-name/);
	});
});
