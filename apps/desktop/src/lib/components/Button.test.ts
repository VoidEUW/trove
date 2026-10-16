import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import Button from './Button.svelte';

const text = (content: string) => createRawSnippet(() => ({ render: () => `<span>${content}</span>` }));
const icon = createRawSnippet(() => ({ render: () => '<svg data-testid="icon"></svg>' }));

function setup(props: Record<string, unknown> = {}) {
	return render(Button, { props: { children: text('Save'), ...props } });
}

describe('ui/button', () => {
	it('is a button of type button, named by its text, with the soft look', () => {
		setup();

		const button = screen.getByRole('button', { name: 'Save' });
		expect(button).toHaveAttribute('type', 'button');
		expect(button).toHaveClass('look-soft', 'neutral', 'md');
	});

	it.each(['neutral', 'primary', 'danger', 'ghost'] as const)('has the %s variant as a class', (variant) => {
		setup({ variant });

		expect(screen.getByRole('button')).toHaveClass(variant);
	});

	it('can be a submit button tied to a form', () => {
		setup({ type: 'submit', form: 'item-form' });

		expect(screen.getByRole('button')).toHaveAttribute('type', 'submit');
		expect(screen.getByRole('button')).toHaveAttribute('form', 'item-form');
	});

	it('runs onclick with the click', async () => {
		const user = userEvent.setup();
		const onclick = vi.fn();
		setup({ onclick });

		await user.click(screen.getByRole('button'));

		expect(onclick).toHaveBeenCalledOnce();
	});

	it('does nothing when disabled', async () => {
		const user = userEvent.setup();
		const onclick = vi.fn();
		setup({ disabled: true, onclick });

		await user.click(screen.getByRole('button'));

		expect(screen.getByRole('button')).toBeDisabled();
		expect(onclick).not.toHaveBeenCalled();
	});

	describe('loading', () => {
		it('is busy, shows a spinner instead of the icon, and ignores clicks', async () => {
			const user = userEvent.setup();
			const onclick = vi.fn();
			setup({ loading: true, icon, onclick });

			await user.click(screen.getByRole('button'));

			expect(screen.getByRole('button')).toHaveAttribute('aria-busy', 'true');
			expect(screen.queryByTestId('icon')).not.toBeInTheDocument();
			expect(onclick).not.toHaveBeenCalled();
		});

		it('is not marked busy otherwise', () => {
			setup({ icon });

			expect(screen.getByRole('button')).not.toHaveAttribute('aria-busy');
			expect(screen.getByTestId('icon')).toBeInTheDocument();
		});
	});

	describe('icon only', () => {
		it('takes its accessible name from the label', () => {
			setup({ children: undefined, icon, label: 'Close' });

			const button = screen.getByRole('button', { name: 'Close' });
			expect(button).toHaveClass('icon-only');
		});

		it('uses the text, not the label, when there is text', () => {
			setup({ label: 'Close' });

			expect(screen.getByRole('button', { name: 'Save' })).not.toHaveAttribute('aria-label');
		});
	});
});
