import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import type { ItemView } from '../bindings/ItemView';
import type { KindView } from '../bindings/KindView';
import en from '../i18n/en.json';
import { addItem, kindName, listItems, listKinds } from './items';

const invokeMock = vi.mocked(invoke);

beforeEach(() => {
	invokeMock.mockReset();
});

function kind(label: KindView['label']): KindView {
	return { id: 'k1', label, counting: 'single', container: false };
}

describe('the item commands', () => {
	it('call the commands the shell registers, with the draft wrapped as the shell expects', async () => {
		invokeMock.mockResolvedValue([]);
		await listItems();
		await listKinds();
		expect(invokeMock.mock.calls).toEqual([['list_items', undefined], ['list_kinds', undefined]]);

		const draft = { name: 'Lamp', kind: 'k1', quantity: 2 };
		invokeMock.mockResolvedValue({ id: 'i1' } as ItemView);
		await expect(addItem(draft)).resolves.toEqual({ id: 'i1' });
		expect(invokeMock).toHaveBeenLastCalledWith('add_item', { draft });
	});
});

describe('kindName', () => {
	it('translates a kind that ships with Trove', () => {
		expect(kindName(kind({ type: 'key', value: 'kind.tools' }))).toBe(en['kind.tools']);
	});

	it('shows a name the user typed exactly as typed', () => {
		expect(kindName(kind({ type: 'text', value: 'kind.tools' }))).toBe('kind.tools');
		expect(kindName(kind({ type: 'text', value: '  Pet supplies ' }))).toBe('  Pet supplies ');
	});

	it('shows a key with no text as it is, so a missing translation is visible', () => {
		expect(kindName(kind({ type: 'key', value: 'kind.not_there' }))).toBe('kind.not_there');
	});
});
