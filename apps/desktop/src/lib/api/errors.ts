// What a failed command looks like to the UI, and the text to show for it (ADR 4, ADR 15).
import type { AppError } from '../bindings/AppError';
import type { ErrorCode } from '../bindings/ErrorCode';
import { t, type MessageKey } from '../i18n/index.svelte';

/**
 * The text key of every error code. A code without an entry, or a key missing from `en.json`, fails
 * `npm run check`.
 */
export const errorKeys = {
	storage: 'error.storage',
	database_too_new: 'error.database_too_new',
	invalid: 'error.invalid',
	name_required: 'item-form.name.required',
	quantity_invalid: 'item-form.quantity.invalid',
	unknown_kind: 'error.unknown_kind',
	internal: 'error.internal'
} as const satisfies Record<ErrorCode, MessageKey>;

/** The typed error a rejected command becomes. `message` is English text for logs, not for users. */
export class CommandError extends Error {
	readonly code: ErrorCode;

	constructor(code: ErrorCode, message: string) {
		super(message);
		this.name = 'CommandError';
		this.code = code;
	}
}

function isAppError(value: unknown): value is AppError {
	return (
		typeof value === 'object' &&
		value !== null &&
		'code' in value &&
		'message' in value &&
		typeof value.code === 'string' &&
		value.code in errorKeys &&
		typeof value.message === 'string'
	);
}

/** Anything that is not an `AppError` from Rust (a missing runtime, a bug) becomes `internal`. */
export function toCommandError(error: unknown): CommandError {
	if (error instanceof CommandError) return error;
	if (isAppError(error)) return new CommandError(error.code, error.message);
	return new CommandError('internal', error instanceof Error ? error.message : String(error));
}

/** The text to show the user for a failed command, in their language. */
export function errorMessage(error: unknown): string {
	return t(errorKeys[toCommandError(error).code]);
}
