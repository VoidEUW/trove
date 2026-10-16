import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }));

import { listen } from '@tauri-apps/api/event';
import { onItemsChanged, onLayoutChanged } from './events';

const listenMock = vi.mocked(listen);

beforeEach(() => {
	listenMock.mockReset();
});

describe.each([
	['items_changed', onItemsChanged, { ids: ['i1'] }],
	['layout_changed', onLayoutChanged, { ids: ['r1'] }]
] as const)('the %s event', (name, subscribe, payload) => {
	it('hands the payload, not the whole Tauri event, to the handler', async () => {
		const unlisten = vi.fn();
		listenMock.mockResolvedValue(unlisten);
		const handler = vi.fn();

		const stop = await subscribe(handler);

		expect(listenMock).toHaveBeenCalledWith(name, expect.any(Function));
		listenMock.mock.calls[0][1]({ event: name, id: 1, payload });
		expect(handler).toHaveBeenCalledExactlyOnceWith(payload);
		expect(stop).toBe(unlisten);
	});
});
