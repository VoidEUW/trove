import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Menu, { type MenuEntry } from './Menu.svelte';

function setup(entries: MenuEntry[], props: Record<string, unknown> = {}) {
	const onclose = vi.fn();
	const view = render(Menu, { props: { label: 'Item actions', entries, onclose, ...props } });
	return { onclose, ...view };
}

const entries = (onselect = vi.fn()): MenuEntry[] => [
	{ type: 'heading', label: 'Item' },
	{ id: 'open', label: 'Open', shortcut: '⌘O', onselect },
	{ id: 'rename', label: 'Rename' },
	{ type: 'separator' },
	{ id: 'delete', label: 'Delete', danger: true }
];

const labels = () => screen.getAllByRole('menuitem').map((item) => item.textContent?.trim());

describe('ui/menu', () => {
	it('is a named menu with its items, a heading and a shortcut hint', () => {
		setup(entries());

		const menu = screen.getByRole('menu', { name: 'Item actions' });
		expect(menu).toHaveClass('look-float');
		expect(labels()).toEqual(['Open ⌘O', 'Rename', 'Delete']);
		expect(within(menu).getByText('Item')).toBeInTheDocument();
		expect(screen.getByRole('menuitem', { name: /Delete/ })).toHaveClass('danger');
	});

	describe('focus and keys', () => {
		it('puts focus on the first item when it opens', () => {
			setup(entries());

			expect(screen.getByRole('menuitem', { name: /Open/ })).toHaveFocus();
		});

		it('puts focus on the checked item when there is one', () => {
			setup([
				{ id: 'a', label: 'Small', checked: false, radio: true },
				{ id: 'b', label: 'Large', checked: true, radio: true }
			]);

			expect(screen.getByRole('menuitemradio', { name: /Large/ })).toHaveFocus();
		});

		it('moves with the arrow keys and wraps around at both ends', async () => {
			const user = userEvent.setup();
			setup(entries());

			await user.keyboard('{ArrowDown}');
			expect(screen.getByRole('menuitem', { name: /Rename/ })).toHaveFocus();
			await user.keyboard('{ArrowDown}{ArrowDown}');
			expect(screen.getByRole('menuitem', { name: /Open/ })).toHaveFocus();
			await user.keyboard('{ArrowUp}');
			expect(screen.getByRole('menuitem', { name: /Delete/ })).toHaveFocus();
		});

		it('jumps to the first and last item with Home and End', async () => {
			const user = userEvent.setup();
			setup(entries());

			await user.keyboard('{End}');
			expect(screen.getByRole('menuitem', { name: /Delete/ })).toHaveFocus();
			await user.keyboard('{Home}');
			expect(screen.getByRole('menuitem', { name: /Open/ })).toHaveFocus();
		});

		it('skips a disabled item', async () => {
			const user = userEvent.setup();
			setup([
				{ id: 'a', label: 'First' },
				{ id: 'b', label: 'Second', disabled: true },
				{ id: 'c', label: 'Third' }
			]);

			await user.keyboard('{ArrowDown}');

			expect(screen.getByRole('menuitem', { name: 'Third' })).toHaveFocus();
		});

		it('closes on Escape and on Tab', async () => {
			const user = userEvent.setup();
			const { onclose } = setup(entries());

			await user.keyboard('{Escape}');
			expect(onclose).toHaveBeenCalledTimes(1);
			await user.keyboard('{Tab}');
			expect(onclose).toHaveBeenCalledTimes(2);
		});

		it('ignores other keys', async () => {
			const user = userEvent.setup();
			const { onclose } = setup(entries());

			await user.keyboard('a{ArrowRight}{ArrowLeft}');

			expect(onclose).not.toHaveBeenCalled();
			expect(screen.getByRole('menuitem', { name: /Open/ })).toHaveFocus();
		});
	});

	describe('choosing', () => {
		it('runs onselect and then closes', async () => {
			const user = userEvent.setup();
			const onselect = vi.fn();
			const { onclose } = setup(entries(onselect));

			await user.click(screen.getByRole('menuitem', { name: /Open/ }));

			expect(onselect).toHaveBeenCalledOnce();
			expect(onclose).toHaveBeenCalledOnce();
		});

		it('does nothing for a disabled item', async () => {
			const user = userEvent.setup();
			const onselect = vi.fn();
			const { onclose } = setup([{ id: 'a', label: 'Archive', disabled: true, onselect }]);

			await user.click(screen.getByRole('menuitem', { name: 'Archive' }));

			expect(onselect).not.toHaveBeenCalled();
			expect(onclose).not.toHaveBeenCalled();
		});

		it('is a checkbox or radio item when it has a checked state', () => {
			setup([
				{ id: 'a', label: 'Archived', checked: true },
				{ id: 'b', label: 'Compact', checked: false, radio: true }
			]);

			expect(screen.getByRole('menuitemcheckbox', { name: /Archived/ })).toHaveAttribute('aria-checked', 'true');
			expect(screen.getByRole('menuitemradio', { name: /Compact/ })).toHaveAttribute('aria-checked', 'false');
		});
	});

	describe('clicking outside', () => {
		it('closes the menu', async () => {
			const user = userEvent.setup();
			const { onclose } = setup(entries());

			await user.click(document.body);

			expect(onclose).toHaveBeenCalledOnce();
		});

		it('does not close it when the click is on the item inside', async () => {
			const user = userEvent.setup();
			const { onclose } = setup([{ id: 'a', label: 'Inside', disabled: true }]);

			await user.click(screen.getByRole('menu'));

			expect(onclose).not.toHaveBeenCalled();
		});

		it('does not close it when the click is on the element that opened it', async () => {
			const user = userEvent.setup();
			const anchor = document.createElement('button');
			document.body.append(anchor);
			const { onclose } = setup(entries(), { anchor });

			await user.click(anchor);

			expect(onclose).not.toHaveBeenCalled();
			anchor.remove();
		});

		it('stops listening once it is gone', async () => {
			const user = userEvent.setup();
			const { onclose, unmount } = setup(entries());
			unmount();

			await user.click(document.body);

			expect(onclose).not.toHaveBeenCalled();
		});
	});

	describe('submenus', () => {
		const withSubmenu = (inner = vi.fn()): MenuEntry[] => [
			{ id: 'sort', label: 'Sort by', submenu: [{ id: 'name', label: 'Name', onselect: inner }] },
			{ id: 'other', label: 'Other' }
		];

		it('is announced as a collapsed menu popup', () => {
			setup(withSubmenu());

			const item = screen.getByRole('menuitem', { name: /Sort by/ });
			expect(item).toHaveAttribute('aria-haspopup', 'menu');
			expect(item).toHaveAttribute('aria-expanded', 'false');
		});

		it('opens with ArrowRight and closes with ArrowLeft, returning focus to its item', async () => {
			const user = userEvent.setup();
			const { onclose } = setup(withSubmenu());

			await user.keyboard('{ArrowRight}');
			const submenu = screen.getByRole('menu', { name: 'Sort by' });
			expect(screen.getByRole('menuitem', { name: /Sort by/ })).toHaveAttribute('aria-expanded', 'true');
			expect(within(submenu).getByRole('menuitem', { name: 'Name' })).toHaveFocus();

			await user.keyboard('{ArrowLeft}');
			expect(screen.queryByRole('menu', { name: 'Sort by' })).not.toBeInTheDocument();
			expect(screen.getByRole('menuitem', { name: /Sort by/ })).toHaveFocus();
			expect(onclose).not.toHaveBeenCalled();
		});

		it('opens on click, and choosing inside runs it and closes the submenu', async () => {
			const user = userEvent.setup();
			const inner = vi.fn();
			setup(withSubmenu(inner));

			await user.click(screen.getByRole('menuitem', { name: /Sort by/ }));
			await user.click(within(screen.getByRole('menu', { name: 'Sort by' })).getByRole('menuitem', { name: 'Name' }));

			expect(inner).toHaveBeenCalledOnce();
			expect(screen.queryByRole('menu', { name: 'Sort by' })).not.toBeInTheDocument();
		});

		it('opens when the pointer enters the item and closes when it enters another', async () => {
			const user = userEvent.setup();
			setup(withSubmenu());

			await user.hover(screen.getByRole('menuitem', { name: /Sort by/ }));
			expect(screen.getByRole('menu', { name: 'Sort by' })).toBeInTheDocument();
			await user.hover(screen.getByRole('menuitem', { name: 'Other' }));
			expect(screen.queryByRole('menu', { name: 'Sort by' })).not.toBeInTheDocument();
		});

		it('ArrowRight on an item without a submenu does nothing', async () => {
			const user = userEvent.setup();
			setup(withSubmenu());

			await user.keyboard('{ArrowDown}{ArrowRight}');

			expect(screen.queryByRole('menu', { name: 'Sort by' })).not.toBeInTheDocument();
		});
	});
});
