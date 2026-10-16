import { render, screen, waitFor, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import Select from './Select.svelte';

const options = [
	{ value: 'tools', label: 'Tools' },
	{ value: 'furniture', label: 'Furniture' },
	{ value: 'other', label: 'Other' }
];

function setup(props: Record<string, unknown> = {}) {
	return render(Select, { props: { label: 'Kind', options, ...props } });
}

const trigger = () => screen.getByRole('button', { name: /Kind/ });

describe('ui/select', () => {
	it('shows the label of the chosen value, or the placeholder while nothing is chosen', () => {
		setup({ value: 'furniture' });
		expect(trigger()).toHaveTextContent('Furniture');
	});

	it('shows the placeholder, dimmed, while nothing is chosen', () => {
		setup({ placeholder: 'Choose a kind' });

		expect(trigger()).toHaveTextContent('Choose a kind');
		expect(screen.getByText('Choose a kind')).toHaveClass('placeholder');
	});

	it('is named by its label and its value together, and announces a menu popup', () => {
		setup({ value: 'tools' });

		expect(trigger()).toHaveAccessibleName('Kind Tools');
		expect(trigger()).toHaveAttribute('aria-haspopup', 'menu');
		expect(trigger()).toHaveAttribute('aria-expanded', 'false');
	});

	describe('opening and choosing', () => {
		it('opens a menu of the options with the current one checked, on click', async () => {
			const user = userEvent.setup();
			setup({ value: 'furniture' });

			await user.click(trigger());

			const menu = screen.getByRole('menu', { name: 'Kind' });
			expect(trigger()).toHaveAttribute('aria-expanded', 'true');
			expect(within(menu).getAllByRole('menuitemradio').map((item) => item.getAttribute('aria-checked'))).toEqual([
				'false',
				'true',
				'false'
			]);
			expect(within(menu).getByRole('menuitemradio', { name: /Furniture/ })).toHaveFocus();
		});

		it('opens with ArrowDown', async () => {
			const user = userEvent.setup();
			setup();
			trigger().focus();

			await user.keyboard('{ArrowDown}');

			expect(screen.getByRole('menu', { name: 'Kind' })).toBeInTheDocument();
		});

		it('closes again when the trigger is clicked a second time', async () => {
			const user = userEvent.setup();
			setup();

			await user.click(trigger());
			await user.click(trigger());

			expect(screen.queryByRole('menu')).not.toBeInTheDocument();
		});

		it('sets the value, closes the menu and returns focus to the trigger when an option is chosen', async () => {
			const user = userEvent.setup();
			setup({ value: 'tools' });

			await user.click(trigger());
			await user.click(screen.getByRole('menuitemradio', { name: /Other/ }));

			expect(trigger()).toHaveTextContent('Other');
			expect(screen.queryByRole('menu')).not.toBeInTheDocument();
			expect(trigger()).toHaveFocus();
		});

		it('chooses with the keyboard', async () => {
			const user = userEvent.setup();
			setup({ value: 'tools' });
			trigger().focus();

			await user.keyboard('{ArrowDown}{ArrowDown}{Enter}');

			expect(trigger()).toHaveTextContent('Furniture');
		});

		it('closes on Escape without changing the value, and returns focus to the trigger', async () => {
			const user = userEvent.setup();
			setup({ value: 'tools' });

			await user.click(trigger());
			await user.keyboard('{Escape}');

			expect(screen.queryByRole('menu')).not.toBeInTheDocument();
			expect(trigger()).toHaveTextContent('Tools');
			expect(trigger()).toHaveFocus();
		});

		it('closes when focus leaves the select', async () => {
			const user = userEvent.setup();
			render(Select, { props: { label: 'Kind', options } });
			const outside = document.createElement('input');
			document.body.append(outside);

			await user.click(trigger());
			outside.focus();

			await waitFor(() => expect(screen.queryByRole('menu')).not.toBeInTheDocument());
			outside.remove();
		});
	});

	it('cannot be opened when disabled', async () => {
		const user = userEvent.setup();
		setup({ disabled: true });

		await user.click(trigger());

		expect(trigger()).toBeDisabled();
		expect(screen.queryByRole('menu')).not.toBeInTheDocument();
	});

	describe('help and error', () => {
		it('describes the trigger with the help', () => {
			setup({ help: 'What it is' });

			expect(trigger()).toHaveAccessibleDescription('What it is');
			expect(trigger()).not.toHaveClass('invalid');
		});

		it('shows the error instead of the help, and marks the trigger', () => {
			setup({ help: 'What it is', error: 'Choose a kind' });

			expect(trigger()).toHaveAccessibleDescription('Choose a kind');
			expect(screen.queryByText('What it is')).not.toBeInTheDocument();
			expect(trigger()).toHaveClass('invalid');
		});

		it('has no description without either', () => {
			setup();

			expect(trigger()).not.toHaveAttribute('aria-describedby');
		});
	});
});
