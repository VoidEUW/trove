import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import StatePage from './StatePage.svelte';

function setup(props: Record<string, unknown> = {}) {
	return render(StatePage, { props: { title: 'Nothing here yet', ...props } });
}

describe('ui/state-page', () => {
	it('shows a title with the soft look, and an empty state by default', () => {
		setup();

		expect(screen.getByRole('heading', { name: 'Nothing here yet' })).toBeInTheDocument();
		expect(document.querySelector('section')).toHaveClass('look-soft', 'empty');
	});

	it.each(['empty', 'error', 'offline', 'not-found'] as const)('has the %s kind as a class', (kind) => {
		setup({ kind });

		expect(document.querySelector('section')).toHaveClass(kind);
	});

	it('announces an error politely and other kinds not at all', () => {
		const { unmount } = setup({ kind: 'error' });
		expect(document.querySelector('section')).toHaveAttribute('aria-live', 'polite');
		unmount();

		setup({ kind: 'empty' });
		expect(document.querySelector('section')).not.toHaveAttribute('aria-live');
	});

	it('shows the description when there is one', () => {
		setup({ description: 'Add the first thing' });

		expect(screen.getByText('Add the first thing')).toBeInTheDocument();
	});

	describe('actions', () => {
		it('runs the primary and secondary actions, the secondary first', async () => {
			const user = userEvent.setup();
			const primary = vi.fn();
			const secondary = vi.fn();
			setup({
				primary: { label: 'Add item', onclick: primary },
				secondary: { label: 'Back', onclick: secondary }
			});

			expect(screen.getAllByRole('button').map((button) => button.textContent?.trim())).toEqual(['Back', 'Add item']);
			expect(screen.getByRole('button', { name: 'Add item' })).toHaveClass('primary');
			await user.click(screen.getByRole('button', { name: 'Add item' }));
			await user.click(screen.getByRole('button', { name: 'Back' }));

			expect(primary).toHaveBeenCalledOnce();
			expect(secondary).toHaveBeenCalledOnce();
		});

		it('shows no buttons without actions', () => {
			setup();

			expect(screen.queryByRole('button')).not.toBeInTheDocument();
		});
	});

	describe('details', () => {
		it('folds technical details away under their label', () => {
			setup({ details: 'storage: disk full', detailsLabel: 'Details' });

			const details = screen.getByText('Details').closest('details');
			expect(details).not.toHaveAttribute('open');
			expect(within(details as HTMLElement).getByText('storage: disk full')).toBeInTheDocument();
		});

		it('opens them on click', async () => {
			const user = userEvent.setup();
			setup({ details: 'storage: disk full', detailsLabel: 'Details' });

			await user.click(screen.getByText('Details'));

			expect(screen.getByText('Details').closest('details')).toHaveAttribute('open');
		});

		it('shows nothing without details', () => {
			setup();

			expect(document.querySelector('details')).toBeNull();
		});
	});
});
