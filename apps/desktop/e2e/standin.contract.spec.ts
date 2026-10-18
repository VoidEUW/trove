// The stand-in must answer the commands the way `trove-desktop` does, or the smoke tests in
// `items.spec.ts` pass against behaviour that no longer exists (ADR 17, tier 3).
// `src/test/standin.contract.test.ts` owns the command list and pins the shapes to the generated
// bindings; here the stand-in itself is run and its answers compared with that contract.
import { expect, test, type Page } from '@playwright/test';
import { contractShape, refusals, shapeOf } from '../src/test/standin-contract';

test.beforeEach(async ({ page }) => {
	await page.addInitScript({ path: 'e2e/tauri-mock.js' });
});

/** Calls a command in the page, the way the API layer would, and brings back whatever it answered. */
function invoke(page: Page, command: string, args?: Record<string, unknown>): Promise<unknown> {
	return page.evaluate(
		async ([name, arguments_]) => {
			const internals = (
				window as unknown as {
					__TAURI_INTERNALS__: {
						invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
					};
				}
			).__TAURI_INTERNALS__;
			return internals.invoke(name, arguments_ ?? undefined);
		},
		[command, args ?? null] as [string, Record<string, unknown> | null]
	);
}

/** The code a command rejected with, or the marker for a promise that resolved. */
async function codeOf(page: Page, command: string, args: Record<string, unknown>) {
	return page.evaluate(
		async ([name, arguments_]) => {
			const internals = (
				window as unknown as {
					__TAURI_INTERNALS__: {
						invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
					};
				}
			).__TAURI_INTERNALS__;
			try {
				await internals.invoke(name, arguments_ ?? undefined);
				return 'resolved';
			} catch (thrown) {
				return (thrown as { code?: string }).code ?? 'no code';
			}
		},
		[command, args] as [string, Record<string, unknown>]
	);
}

test('answers every command the API layer wraps, with the fields the bindings declare', async ({
	page
}) => {
	await page.goto('/items');

	// A row first, so list_items and add_item have something to answer with.
	const [kind] = (await invoke(page, 'list_kinds')) as { id: string }[];
	await invoke(page, 'add_item', { draft: { name: 'Floor lamp', kind: kind!.id, quantity: 1 } });

	expect(shapeOf(await invoke(page, 'app_info'))).toEqual(contractShape('app_info'));
	expect(shapeOf(await invoke(page, 'list_items'))).toEqual(contractShape('list_items'));
	expect(shapeOf(await invoke(page, 'list_kinds'))).toEqual(contractShape('list_kinds'));
	expect(shapeOf(await invoke(page, 'add_item', {
		draft: { name: 'Drill', kind: kind!.id, quantity: 1 }
	}))).toEqual(contractShape('add_item'));
});

test('rejects only with codes the shell can send', async ({ page }) => {
	await page.goto('/items');

	const kind = '00000000-0000-7000-8000-000000000001';
	const codes = [
		await codeOf(page, 'add_item', { draft: { name: '   ', kind, quantity: 1 } }),
		await codeOf(page, 'add_item', { draft: { name: 'Drill', kind, quantity: 0 } }),
		await codeOf(page, 'add_item', { draft: { name: 'Drill', kind: 'nope', quantity: 1 } }),
		await codeOf(page, 'no_such_command', {})
	];

	expect([...codes].sort()).toEqual([...refusals].sort());
});

test('reports the version this package.json declares', async ({ page }) => {
	await page.goto('/items');

	const info = (await invoke(page, 'app_info')) as { version: string };
	const { version } = JSON.parse(
		(await import('node:fs')).readFileSync(new URL('../package.json', import.meta.url), 'utf8')
	) as { version: string };

	expect(info.version).toBe(version);
});