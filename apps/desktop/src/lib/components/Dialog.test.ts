import { render, screen, waitFor, within } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import Dialog from './Dialog.svelte';

const snippet = (html: string) => createRawSnippet(() => ({ render: () => html }));

function setup(props: Record<string, unknown> = {}) {
	return render(Dialog, {
		props: { open: true, title: 'Add an item', children: snippet('<p>Body text</p>'), ...props }
	});
}

const dialog = () => document.querySelector('dialog') as HTMLDialogElement;

describe('ui/dialog', () => {
	it('is a modal dialog named by its title, with the floating look', () => {
		setup();

		expect(screen.getByRole('dialog', { name: 'Add an item' })).toBe(dialog());
		expect(dialog()).toHaveClass('look-float');
		expect(screen.getByRole('heading', { level: 2, name: 'Add an item' })).toBeInTheDocument();
		expect(screen.getByText('Body text')).toBeInTheDocument();
	});

	it('is described by its description when there is one, and by nothing otherwise', () => {
		const { unmount } = setup({ description: 'Add something you own' });
		expect(screen.getByRole('dialog')).toHaveAccessibleDescription('Add something you own');
		unmount();

		setup();
		expect(screen.getByRole('dialog')).not.toHaveAttribute('aria-describedby');
	});

	it('shows the footer actions only when given', () => {
		const { unmount } = setup({ actions: snippet('<button type="button">Save</button>') });
		const footer = document.querySelector('footer') as HTMLElement;
		expect(within(footer).getByRole('button', { name: 'Save' })).toBeInTheDocument();
		unmount();

		setup();
		expect(document.querySelector('footer')).toBeNull();
	});

	describe('opening and closing', () => {
		it('stays closed while open is false', () => {
			setup({ open: false });

			expect(dialog()).not.toHaveAttribute('open');
			expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
		});

		it('opens and closes when open changes', async () => {
			const { rerender } = setup({ open: false });

			await rerender({ open: true });
			await waitFor(() => expect(dialog()).toHaveAttribute('open'));

			await rerender({ open: false });
			await waitFor(() => expect(dialog()).not.toHaveAttribute('open'));
		});

		it('tells the owner it closed, however it was closed (for example with Escape)', async () => {
			const onclose = vi.fn();
			setup({ onclose });

			dialog().close();

			await waitFor(() => expect(onclose).toHaveBeenCalledOnce());
		});

		it('can be closed and opened again', async () => {
			const onclose = vi.fn();
			const { rerender } = setup({ onclose });
			dialog().close();
			await waitFor(() => expect(onclose).toHaveBeenCalledOnce());

			await rerender({ open: true });

			await waitFor(() => expect(dialog()).toHaveAttribute('open'));
		});
	});

	describe('settings layout', () => {
		it('puts the sidebar to the left of the body', () => {
			setup({ sidebar: snippet('<nav>Sections</nav>') });

			expect(dialog()).toHaveClass('settings');
			expect(document.querySelector('aside')).toContainElement(screen.getByText('Sections'));
		});

		it('has no sidebar and no settings layout otherwise', () => {
			setup();

			expect(dialog()).not.toHaveClass('settings');
			expect(document.querySelector('aside')).toBeNull();
		});

		it('replaces the body when the pane changes, so the new pane can fade in', async () => {
			const { rerender } = setup({ sidebar: snippet('<nav>Sections</nav>'), paneKey: 'general' });
			const before = document.querySelector('.body');

			await rerender({ paneKey: 'appearance' });

			expect(document.querySelector('.body')).not.toBe(before);
			expect(screen.getByText('Body text')).toBeInTheDocument();
		});
	});
});
