import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';
import Topbar from './Topbar.svelte';

describe('ui/topbar', () => {
	it('shows the title of the screen as the page heading, with the open look', () => {
		render(Topbar, { props: { title: 'All items' } });

		expect(screen.getByRole('heading', { level: 1, name: 'All items' })).toBeInTheDocument();
		expect(screen.getByRole('banner')).toHaveClass('look-open');
	});

	it('shows the toolbar on the right when there is one', () => {
		const toolbar = createRawSnippet(() => ({ render: () => '<button type="button">Add item</button>' }));
		render(Topbar, { props: { title: 'All items', toolbar } });

		expect(screen.getByRole('button', { name: 'Add item' })).toBeInTheDocument();
	});

	it('has no toolbar area without one', () => {
		render(Topbar, { props: { title: 'All items' } });

		expect(document.querySelector('.toolbar')).toBeNull();
	});

	it('follows a new title', async () => {
		const { rerender } = render(Topbar, { props: { title: 'All items' } });

		await rerender({ title: 'Settings' });

		expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Settings');
	});
});
