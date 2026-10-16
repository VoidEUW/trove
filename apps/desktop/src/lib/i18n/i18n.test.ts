import fc from 'fast-check';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import en from './en.json';
import { getLocale, locales, setLocale, t, tStored, type MessageKey } from './index.svelte';

const greeting = 'test.greeting' as MessageKey;
const onlyEnglish = 'test.only_english' as MessageKey;

beforeEach(() => {
	Object.assign(locales.en, {
		[greeting]: 'Hello {name}, you have {count} items',
		[onlyEnglish]: 'English only'
	});
	locales.de = { [greeting]: 'Hallo {name}, du hast {count} Dinge' } as (typeof locales)[string];
});

afterEach(() => {
	delete (locales.en as Record<string, string>)[greeting];
	delete (locales.en as Record<string, string>)[onlyEnglish];
	delete locales.de;
	setLocale('en');
});

describe('t', () => {
	it('fills {name} placeholders from the parameters', () => {
		setLocale('en');

		expect(t(greeting, { name: 'Ada', count: 3 })).toBe('Hello Ada, you have 3 items');
	});

	it('leaves a placeholder visible when its parameter is missing', () => {
		setLocale('en');

		expect(t(greeting, { name: 'Ada' })).toBe('Hello Ada, you have {count} items');
	});

	it('inserts any text without reading it as a pattern', () => {
		setLocale('en');
		fc.assert(
			fc.property(fc.string(), fc.string(), (name, other) => {
				expect(t(greeting, { name, count: other })).toBe(`Hello ${name}, you have ${other} items`);
			})
		);
	});

	it('returns the English text for a key the locale does not translate', () => {
		setLocale('de');

		expect(t(greeting, { name: 'Ada', count: 3 })).toBe('Hallo Ada, du hast 3 Dinge');
		expect(t(onlyEnglish)).toBe('English only');
	});

	it('has text for every key in en.json', () => {
		setLocale('en');
		for (const key of Object.keys(en) as MessageKey[]) {
			expect(t(key), key).toBe(en[key]);
		}
	});
});

describe('locale matching', () => {
	it.each([
		['de', 'de'],
		['de-AT', 'de'],
		['de-DE', 'de'],
		['en-GB', 'en'],
		['fr', 'en'],
		['', 'en']
	])('%s uses %s', (wanted, used) => {
		setLocale(wanted);

		expect(getLocale()).toBe(used);
	});

	it('sets the language of the document', () => {
		setLocale('de-AT');

		expect(document.documentElement.lang).toBe('de');
	});
});

describe('tStored', () => {
	it('translates a stored message key and leaves anything else as it is', () => {
		setLocale('en');

		expect(tStored('kind.tools')).toBe(en['kind.tools']);
		expect(tStored('Pet supplies')).toBe('Pet supplies');
	});
});
