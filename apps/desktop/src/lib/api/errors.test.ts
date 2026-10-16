import fc from 'fast-check';
import { describe, expect, it } from 'vitest';
import en from '../i18n/en.json';
import { CommandError, errorKeys, errorMessage, toCommandError } from './errors';

describe('toCommandError', () => {
	it('turns an AppError from Rust into a CommandError with the same code and message', () => {
		const error = toCommandError({ code: 'name_required', message: 'an item needs a name' });

		expect(error).toBeInstanceOf(CommandError);
		expect(error.code).toBe('name_required');
		expect(error.message).toBe('an item needs a name');
	});

	it('returns a CommandError as it is', () => {
		const error = new CommandError('storage', 'disk full');

		expect(toCommandError(error)).toBe(error);
	});

	it('makes anything else an internal error that keeps the message', () => {
		expect(toCommandError(new Error('boom'))).toMatchObject({ code: 'internal', message: 'boom' });
		expect(toCommandError('plain text')).toMatchObject({ code: 'internal', message: 'plain text' });
		expect(toCommandError(undefined)).toMatchObject({ code: 'internal', message: 'undefined' });
	});

	it.each([
		['an unknown code', { code: 'no_such_code', message: 'x' }],
		['a missing message', { code: 'storage' }],
		['a message that is not text', { code: 'storage', message: 5 }],
		['a code that is not text', { code: 5, message: 'x' }],
		['null', null]
	])('does not trust an object with %s', (_, value) => {
		expect(toCommandError(value).code).toBe('internal');
	});

	it('copes with an object that has no prototype, which String() cannot convert', () => {
		expect(toCommandError(Object.create(null)).code).toBe('internal');
	});

	it('always returns a known code and never throws, whatever it is given', () => {
		fc.assert(
			fc.property(fc.anything({ withNullPrototype: true }), (value) => {
				expect(Object.keys(errorKeys)).toContain(toCommandError(value).code);
			})
		);
	});

	it('keeps the code of any well-formed AppError', () => {
		fc.assert(
			fc.property(fc.constantFrom(...Object.keys(errorKeys)), fc.string(), (code, message) => {
				expect(toCommandError({ code, message })).toMatchObject({ code, message });
			})
		);
	});
});

describe('errorMessage', () => {
	it.each(Object.entries(errorKeys))('has the text behind %s in en.json', (code, key) => {
		expect(key in en).toBe(true);
		expect(errorMessage({ code, message: 'for the log' })).toBe(en[key]);
	});

	it('shows the internal text for something that is not an AppError', () => {
		expect(errorMessage(new Error('boom'))).toBe(en['error.internal']);
	});

	it('never shows the English log message to the user', () => {
		expect(errorMessage({ code: 'storage', message: 'disk full' })).not.toContain('disk full');
	});
});
