// The only place that calls `invoke`. Each command gets one thin wrapper that uses `call`.
import { invoke } from '@tauri-apps/api/core';
import { toCommandError } from './errors';

/** Calls a command and rejects with a typed `CommandError` instead of a bare value. */
export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	try {
		return await invoke<T>(command, args);
	} catch (error) {
		throw toCommandError(error);
	}
}
