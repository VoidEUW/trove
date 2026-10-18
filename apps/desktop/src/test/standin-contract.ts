// What `e2e/tauri-mock.js` must answer with, one entry per command the UI wraps in `src/lib/api/`.
// The types are the generated bindings, and CI regenerates those from `trove-desktop` and fails when
// they are stale. So a change to a command's result in Rust makes this file stop satisfying
// `StandinResults`, and `npm run check` fails until the stand-in is brought back in line.
// `standin.contract.test.ts` checks the list against the API layer; `e2e/standin.contract.spec.ts`
// checks the stand-in still answers with this shape in a browser.
import type { ErrorCode } from '$lib/bindings/ErrorCode';
import type { AppInfo } from '$lib/bindings/AppInfo';
import type { ItemView } from '$lib/bindings/ItemView';
import type { KindView } from '$lib/bindings/KindView';

/** Every command `src/lib/api/` wraps, and what it returns. */
export interface StandinResults {
	app_info: AppInfo;
	list_items: ItemView[];
	list_kinds: KindView[];
	add_item: ItemView;
}

/** One of each nested view, so a field added or removed in Rust fails `npm run check` here. */
const aKind: KindView = {
	id: '00000000-0000-7000-8000-000000000001',
	label: { type: 'key', value: 'kind.electrical' },
	counting: 'single',
	container: false
};

const anItem: ItemView = {
	id: '0199c3a0-0000-7000-8000-000000000000',
	name: 'Floor lamp',
	kind: aKind,
	quantity: 1,
	created_ms: 0
};

/** The fields of each command's result. Ids and timestamps differ per run, so only names are compared. */
export const expected: StandinResults = {
	app_info: { version: '0.1.0', device_id: '0199c3a0-0000-7000-8000-000000000000' },
	list_items: [anItem],
	list_kinds: [aKind],
	add_item: anItem
};

/** The refusals the stand-in has to be able to produce, as `trove-core` produces them. */
export const refusals = [
	'name_required',
	'quantity_invalid',
	'unknown_kind',
	'internal'
] as const satisfies readonly ErrorCode[];

/** The fields of a command's answer, looking inside a list so an element's shape is what is compared. */
export function shapeOf(value: unknown): string[] {
	return Object.keys((Array.isArray(value) ? value[0] : value) as object).sort();
}

/** The fields of the first element of a command's result, which is what has to be compared. */
export function contractShape(command: keyof StandinResults): string[] {
	return shapeOf(expected[command]);
}