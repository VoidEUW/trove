import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import { CommandError } from './errors';
import { call } from './invoke';

const invokeMock = vi.mocked(invoke);

beforeEach(() => {
	invokeMock.mockReset();
});

describe('call', () => {
	it('passes the command and its arguments to Tauri and returns the result', async () => {
		invokeMock.mockResolvedValue([1, 2]);

		await expect(call<number[]>('list_things', { limit: 2 })).resolves.toEqual([1, 2]);
		expect(invokeMock).toHaveBeenCalledWith('list_things', { limit: 2 });
	});

	it('rejects with a typed CommandError when Rust returns an AppError', async () => {
		invokeMock.mockImplementation(() =>
			Promise.reject({ code: 'unknown_kind', message: 'no such kind' })
		);

		const error = await call('add_item').catch((caught: unknown) => caught);

		expect(error).toBeInstanceOf(CommandError);
		expect((error as CommandError).code).toBe('unknown_kind');
		expect((error as CommandError).message).toBe('no such kind');
	});

	it('rejects with an internal CommandError when the runtime itself fails', async () => {
		invokeMock.mockImplementation(() => Promise.reject(new Error('window is gone')));

		const error = await call('add_item').catch((caught: unknown) => caught);

		expect(error).toBeInstanceOf(CommandError);
		expect((error as CommandError).code).toBe('internal');
		expect((error as CommandError).message).toBe('window is gone');
	});
});
