import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import TextField from './TextField.svelte';

function setup(props: Record<string, unknown> = {}) {
	return render(TextField, { props: { label: 'Name', ...props } });
}

describe('ui/text-field', () => {
	it('is a text box named by its label', () => {
		setup();

		expect(screen.getByRole('textbox', { name: 'Name' })).toHaveClass('look-open');
	});

	it('takes what is typed and reports it', async () => {
		const user = userEvent.setup();
		const oninput = vi.fn();
		setup({ oninput });

		await user.type(screen.getByRole('textbox'), 'Lamp');

		expect(screen.getByRole('textbox')).toHaveValue('Lamp');
		expect(oninput).toHaveBeenCalledTimes(4);
	});

	it('starts with the value it is given', () => {
		setup({ value: 'Drill' });

		expect(screen.getByRole('textbox')).toHaveValue('Drill');
	});

	it('is a multi-line field when asked, with the rows', () => {
		setup({ multiline: true, rows: 5 });

		const field = screen.getByRole('textbox', { name: 'Name' });
		expect(field.tagName).toBe('TEXTAREA');
		expect(field).toHaveAttribute('rows', '5');
	});

	it('passes through the type, placeholder, required, disabled and maxlength', () => {
		setup({ placeholder: 'Floor lamp', required: true, maxlength: 20, name: 'item-name' });

		const field = screen.getByRole('textbox');
		expect(field).toHaveAttribute('placeholder', 'Floor lamp');
		expect(field).toBeRequired();
		expect(field).toHaveAttribute('maxlength', '20');
		expect(field).toHaveAttribute('name', 'item-name');
	});

	it('cannot be typed in when disabled', async () => {
		const user = userEvent.setup();
		setup({ disabled: true });

		await user.type(screen.getByRole('textbox'), 'x');

		expect(screen.getByRole('textbox')).toBeDisabled();
		expect(screen.getByRole('textbox')).toHaveValue('');
	});

	describe('help, error and counter', () => {
		it('describes the field with the help', () => {
			setup({ help: 'What it is called' });

			expect(screen.getByRole('textbox')).toHaveAccessibleDescription('What it is called');
			expect(screen.getByRole('textbox')).not.toHaveAttribute('aria-invalid');
		});

		it('shows the error in place of the help and marks the field invalid', () => {
			setup({ help: 'What it is called', error: 'Give it a name' });

			expect(screen.getByRole('textbox')).toHaveAccessibleDescription('Give it a name');
			expect(screen.getByRole('textbox')).toHaveAttribute('aria-invalid', 'true');
			expect(screen.queryByText('What it is called')).not.toBeInTheDocument();
			expect(screen.getByText('Give it a name')).toHaveClass('error');
		});

		it('has no description without help or error', () => {
			setup();

			expect(screen.getByRole('textbox')).not.toHaveAttribute('aria-describedby');
		});

		it('counts the characters against the maximum while typing', async () => {
			const user = userEvent.setup();
			setup({ maxlength: 10, counter: true });
			expect(screen.getByText('0 / 10')).toBeInTheDocument();

			await user.type(screen.getByRole('textbox'), 'Lamp');

			expect(screen.getByText('4 / 10')).toBeInTheDocument();
		});

		it('does not stop typing at the maximum by itself: the browser does', async () => {
			const user = userEvent.setup();
			setup({ maxlength: 3, counter: true });

			await user.type(screen.getByRole('textbox'), 'abcdef');

			expect(screen.getByRole('textbox')).toHaveValue('abc');
			expect(screen.getByText('3 / 3')).toBeInTheDocument();
		});

		it('shows no counter without a maximum', () => {
			setup({ counter: true });

			expect(screen.queryByText(/ \/ /)).not.toBeInTheDocument();
		});
	});
});
