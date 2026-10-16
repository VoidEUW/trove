import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import { getAppInfo } from './app';

beforeEach(() => {
	vi.mocked(invoke).mockReset();
});

describe('getAppInfo', () => {
	it('asks the shell for app_info and returns it', async () => {
		const info = { name: 'Trove', version: '0.1.0' };
		vi.mocked(invoke).mockResolvedValue(info);

		await expect(getAppInfo()).resolves.toEqual(info);
		expect(invoke).toHaveBeenCalledWith('app_info', undefined);
	});
});
