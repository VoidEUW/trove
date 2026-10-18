import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import { expect, test, type Page } from '@playwright/test';

const axeSource = readFileSync(createRequire(import.meta.url).resolve('axe-core/axe.min.js'), 'utf8');

declare global {
	interface Window {
		__mock: { calls: string[]; failNext: { code: string; message: string } | null };
	}
}

test.beforeEach(async ({ page }) => {
	await page.addInitScript({ path: 'e2e/tauri-mock.js' });
});

async function addItem(page: Page, name: string, quantity = '1') {
	await page.getByRole('button', { name: 'Add item' }).first().click();
	const dialog = page.getByRole('dialog', { name: 'Add an item' });
	await dialog.getByRole('textbox', { name: 'Name' }).fill(name);
	await dialog.getByRole('textbox', { name: 'Quantity' }).fill(quantity);
	await dialog.getByRole('button', { name: 'Save item' }).click();
	return dialog;
}

async function violations(page: Page, { contrast = true } = {}) {
	await page.evaluate(axeSource);
	return page.evaluate(async (checkContrast) => {
		// @ts-expect-error axe is injected by the line above
		const { violations } = await axe.run({ rules: { 'color-contrast': { enabled: checkContrast } } });
		return violations.map(
			(v: { id: string; nodes: { target: string[] }[] }) =>
				`${v.id}: ${v.nodes.map((n) => n.target.join(' ')).join(', ')}`
		);
	}, contrast);
}

/** The theme is chosen by `data-theme` on the root element (dark is the default). */
async function useTheme(page: Page, theme: 'light' | 'dark') {
	await page.emulateMedia({ reducedMotion: 'reduce' });
	await page.goto('/items');
	await page.evaluate((name) => (document.documentElement.dataset.theme = name), theme);
	// The tokens set `color-scheme` per theme, so this proves the theme really changed.
	expect(await page.evaluate(() => getComputedStyle(document.documentElement).colorScheme)).toBe(theme);
}

async function selectNewItem(page: Page) {
	await page.getByRole('button', { name: 'Add item' }).first().click();
	await page.getByRole('dialog').getByRole('textbox', { name: 'Name' }).fill('Floor lamp');
	await page.getByRole('button', { name: 'Save item' }).click();
	const row = page.getByRole('row', { name: /Floor lamp/ });
	await row.click();
	await expect(row).toHaveAttribute('aria-current', 'true');
}

test('opens on the Items screen, empty, with a way to add the first item', async ({ page }) => {
	await page.goto('/');

	await expect(page).toHaveURL(/\/items$/);
	await expect(page.getByRole('heading', { name: 'Nothing here yet' })).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Main' })).toBeVisible();
});

test('adds an item and shows it in the list, still there after a reload', async ({ page }) => {
	await page.goto('/items');

	const dialog = await addItem(page, 'Floor lamp', '2');

	await expect(dialog).toBeHidden();
	const row = page.getByRole('row', { name: /Floor lamp/ });
	await expect(row).toBeVisible();
	await expect(row).toContainText('Electrical');
	await expect(row).toContainText('2');
	await page.reload();
	await expect(page.getByRole('row', { name: /Floor lamp/ })).toBeVisible();
});

test('asks for a name and does not call the shell until there is one', async ({ page }) => {
	await page.goto('/items');
	await page.getByRole('button', { name: 'Add item' }).first().click();
	const dialog = page.getByRole('dialog');

	await dialog.getByRole('button', { name: 'Save item' }).click();

	await expect(dialog.getByText('Give it a name')).toBeVisible();
	expect(await page.evaluate(() => window.__mock.calls.filter((c) => c === 'add_item'))).toEqual([]);
});

test('shows a refusal from the shell and keeps the form open', async ({ page }) => {
	await page.goto('/items');
	await page.getByRole('button', { name: 'Add item' }).first().click();
	const dialog = page.getByRole('dialog');
	await dialog.getByRole('textbox', { name: 'Name' }).fill('Drill');
	await page.evaluate(() => (window.__mock.failNext = { code: 'storage', message: 'disk full' }));

	await dialog.getByRole('button', { name: 'Save item' }).click();

	await expect(dialog.getByRole('alert')).toBeVisible();
	await expect(dialog).toBeVisible();
	await expect(dialog.getByRole('textbox', { name: 'Name' })).toHaveValue('Drill');
});

test('sorts the list by name from the column header', async ({ page }) => {
	await page.goto('/items');
	await addItem(page, 'Screws');
	await expect(page.getByRole('row', { name: /Screws/ })).toBeVisible();
	await addItem(page, 'Drill');
	await expect(page.getByRole('row', { name: /Drill/ })).toBeVisible();

	await page.getByRole('button', { name: 'Name' }).click();

	await expect(page.getByRole('columnheader', { name: 'Name' })).toHaveAttribute('aria-sort', 'ascending');
	const names = await page.getByRole('row').evaluateAll((rows) => rows.slice(1).map((r) => r.textContent ?? ''));
	expect(names[0]).toContain('Drill');
	expect(names[1]).toContain('Screws');
});

// Reduced motion keeps the dialog's entry animation from changing colours mid-check.
for (const theme of ['light', 'dark'] as const) {
	test(`has no accessibility violations in the ${theme} theme: empty, and with the form open`, async ({ page }) => {
		await useTheme(page, theme);
		await expect(page.getByRole('heading', { name: 'Nothing here yet' })).toBeVisible();
		expect(await violations(page)).toEqual([]);

		await page.getByRole('button', { name: 'Add item' }).first().click();
		await expect(page.getByRole('dialog')).toBeVisible();
		expect(await violations(page)).toEqual([]);
	});

	test(`has no structural accessibility violations in the ${theme} theme with a selected row`, async ({ page }) => {
		await useTheme(page, theme);
		await selectNewItem(page);
		expect(await violations(page, { contrast: false })).toEqual([]);
	});

	// With a row selected this also proves the text on the `--accent-soft` selected fill is readable:
	// that fill is the theme's quietest surface for text in the light theme (4.55:1).
	test(`table text meets contrast in the ${theme} theme`, async ({ page }) => {
		await useTheme(page, theme);
		await selectNewItem(page);
		expect(await violations(page)).toEqual([]);
	});
}
