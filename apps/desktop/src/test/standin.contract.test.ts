// The stand-in for the Rust side must not drift from the real commands (ADR 17, tier 3). Without this,
// a command's result or an error code can change in `trove-desktop` while the smoke tests keep
// passing against behaviour that no longer exists.
//
// The halves sit either side of the boundary on purpose:
//   - here, in the type-checked part: the command list is compared with the wrappers `src/lib/api/`
//     actually exports, and the shapes and refusal codes in `standin-contract.ts` are pinned to the
//     generated bindings, so a change in Rust fails `npm run check`.
//   - in `e2e/standin.contract.spec.ts`: the stand-in itself is run and its answers compared, because
//     only there does it exist.
import { describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import * as api from '$lib/api';
import { addItem, getAppInfo, listItems, listKinds } from '$lib/api';
import { expected } from './standin-contract';

const invokeMock = vi.mocked(invoke);

/**
 * The exports of `$lib/api` that do not reach the shell: a pure helper, the two event listeners and
 * the error helpers (`CommandError` is a class, so `typeof` reports it as a function).
 */
const notCommands = [
	'kindName',
	'onItemsChanged',
	'onLayoutChanged',
	'errorMessage',
	'toCommandError',
	'CommandError'
];

describe('the contract between the browser stand-in and the shell', () => {
	it('lists a command for every wrapper in the API layer', () => {
		const wrappers = Object.entries(api)
			.filter(([name, value]) => typeof value === 'function' && !notCommands.includes(name))
			.map(([name]) => name);

		// A new wrapper added without a contract entry fails here, which is the drift we want to see.
		expect(wrappers.sort()).toEqual(['addItem', 'getAppInfo', 'listItems', 'listKinds']);
	});

	it('names the command each wrapper sends', async () => {
		invokeMock.mockResolvedValue([]);

		await getAppInfo();
		await listItems();
		await listKinds();
		await addItem({ name: 'Floor lamp', kind: 'a-kind', quantity: 1 });

		const called = [...new Set(invokeMock.mock.calls.map(([command]) => command))];
		expect(called.sort()).toEqual(Object.keys(expected).sort());
	});
});