import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import fc from 'fast-check';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Harness from '../../test/SidebarHarness.svelte';
import { MAX_WIDTH, MIN_WIDTH } from './Sidebar.svelte';

beforeEach(() => {
	// jsdom has no pointer capture.
	Element.prototype.setPointerCapture = vi.fn();
});

const edge = () => screen.getByRole('separator', { name: 'Resize' });
const aside = () => document.querySelector('aside') as HTMLElement;

describe('ui/sidebar', () => {
	it('has a named navigation holding its entries, and the title', () => {
		render(Harness);

		expect(screen.getByRole('navigation', { name: 'Main' })).toBeInTheDocument();
		expect(screen.getByText('Trove')).toBeInTheDocument();
		expect(aside()).toHaveClass('look-float');
	});

	describe('collapsing', () => {
		it('collapses and expands with the button, which names the action it will do', async () => {
			const user = userEvent.setup();
			render(Harness);
			expect(screen.getByRole('button', { name: 'Collapse' })).toHaveAttribute('aria-expanded', 'true');

			await user.click(screen.getByRole('button', { name: 'Collapse' }));

			expect(aside()).toHaveClass('collapsed');
			expect(screen.getByRole('button', { name: 'Expand' })).toHaveAttribute('aria-expanded', 'false');
			await user.click(screen.getByRole('button', { name: 'Expand' }));
			expect(aside()).not.toHaveClass('collapsed');
		});

		it('hides the resize edge while collapsed', async () => {
			const user = userEvent.setup();
			render(Harness);
			expect(edge()).toBeInTheDocument();

			await user.click(screen.getByRole('button', { name: 'Collapse' }));

			expect(screen.queryByRole('separator')).not.toBeInTheDocument();
		});

		it('starts collapsed when told to, and lets its entries show their label as a title', () => {
			render(Harness, { props: { collapsed: true } });

			expect(aside()).toHaveClass('collapsed');
			expect(screen.getByRole('link', { name: /Items/ })).toHaveAttribute('title', 'Items');
		});
	});

	describe('resizing', () => {
		it('is a focusable vertical separator that reports its width and limits', () => {
			render(Harness);

			expect(edge()).toHaveAttribute('aria-orientation', 'vertical');
			expect(edge()).toHaveAttribute('tabindex', '0');
			expect(edge()).toHaveAttribute('aria-valuenow', '52');
			expect(edge()).toHaveAttribute('aria-valuemin', String(MIN_WIDTH));
			expect(edge()).toHaveAttribute('aria-valuemax', String(MAX_WIDTH));
		});

		it('grows and shrinks by 2 with the arrow keys', async () => {
			const user = userEvent.setup();
			render(Harness);
			edge().focus();

			await user.keyboard('{ArrowRight}{ArrowRight}');
			expect(edge()).toHaveAttribute('aria-valuenow', '56');
			await user.keyboard('{ArrowLeft}');
			expect(edge()).toHaveAttribute('aria-valuenow', '54');
		});

		it('ignores other keys', async () => {
			const user = userEvent.setup();
			render(Harness);
			edge().focus();

			await user.keyboard('{ArrowUp}{ArrowDown}a{Home}');

			expect(edge()).toHaveAttribute('aria-valuenow', '52');
		});

		it('never goes outside the limits, however many keys are pressed', async () => {
			const user = userEvent.setup();
			render(Harness, { props: { width: MAX_WIDTH - 1 } });
			edge().focus();

			await user.keyboard('{ArrowRight}{ArrowRight}{ArrowRight}');
			expect(edge()).toHaveAttribute('aria-valuenow', String(MAX_WIDTH));
			await user.keyboard('{ArrowLeft>40/}');
			expect(edge()).toHaveAttribute('aria-valuenow', String(MIN_WIDTH));
		});

		it('keeps the width inside the limits for any sequence of arrow keys', async () => {
			render(Harness);
			edge().focus();
			const keys = fc.array(fc.constantFrom('ArrowLeft', 'ArrowRight'), { maxLength: 40 });

			await fc.assert(
				fc.asyncProperty(keys, async (sequence) => {
					for (const key of sequence) await fireEvent.keyDown(edge(), { key });
					const width = Number(edge().getAttribute('aria-valuenow'));
					expect(width).toBeGreaterThanOrEqual(MIN_WIDTH);
					expect(width).toBeLessThanOrEqual(MAX_WIDTH);
				}),
				{ numRuns: 30 }
			);
		});

		describe('dragging', () => {
			function drag(clientX: number) {
				aside().style.setProperty('--space', '8px');
				return fireEvent.pointerMove(edge(), { clientX });
			}

			it('follows the pointer while it is held, in multiples of the spacing unit', async () => {
				render(Harness);
				await fireEvent.pointerDown(edge(), { pointerId: 1 });

				await drag(8 * 60);

				expect(edge()).toHaveAttribute('aria-valuenow', '60');
				expect(aside()).toHaveClass('dragging');
			});

			it('stops following the pointer after it is released or cancelled', async () => {
				render(Harness);
				await fireEvent.pointerDown(edge(), { pointerId: 1 });
				await drag(8 * 60);

				await fireEvent.pointerUp(edge());
				await drag(8 * 70);
				expect(edge()).toHaveAttribute('aria-valuenow', '60');
				expect(aside()).not.toHaveClass('dragging');

				await fireEvent.pointerDown(edge(), { pointerId: 1 });
				await fireEvent.pointerCancel(edge());
				await drag(8 * 70);
				expect(edge()).toHaveAttribute('aria-valuenow', '60');
			});

			it('does not move without the pointer being held', async () => {
				render(Harness);

				await drag(8 * 70);

				expect(edge()).toHaveAttribute('aria-valuenow', '52');
			});

			it('stays within the limits however far the pointer goes', async () => {
				render(Harness);
				await fireEvent.pointerDown(edge(), { pointerId: 1 });

				await drag(8 * 500);
				expect(edge()).toHaveAttribute('aria-valuenow', String(MAX_WIDTH));
				await drag(-100);
				expect(edge()).toHaveAttribute('aria-valuenow', String(MIN_WIDTH));
			});
		});
	});
});

describe('ui/sidebar-entry', () => {
	it('is a link with an href, and marks the active page', () => {
		render(Harness);

		const link = screen.getByRole('link', { name: /Items/ });
		expect(link).toHaveAttribute('href', '/items');
		expect(link).toHaveAttribute('aria-current', 'page');
		expect(screen.getByText('3')).toBeInTheDocument();
	});

	it('is a button without an href, and runs onclick', async () => {
		const user = userEvent.setup();
		const onselect = vi.fn();
		render(Harness, { props: { onselect } });

		await user.click(screen.getByRole('button', { name: 'Settings' }));

		expect(onselect).toHaveBeenCalledOnce();
		expect(screen.getByRole('button', { name: 'Settings' })).not.toHaveAttribute('aria-current');
	});

	it('does nothing when disabled, and a disabled link is not a link', async () => {
		const user = userEvent.setup();
		const onselect = vi.fn();
		render(Harness, { props: { onselect } });

		await user.click(screen.getByRole('button', { name: 'Soon' }));

		expect(onselect).not.toHaveBeenCalled();
		expect(screen.getByRole('button', { name: 'Soon' })).toBeDisabled();
		expect(screen.queryByRole('link', { name: 'Hidden link' })).not.toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Hidden link' })).toBeDisabled();
	});
});
