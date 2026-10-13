import type { ItemDraftInput } from '../bindings/ItemDraftInput';
import type { ItemView } from '../bindings/ItemView';
import type { KindView } from '../bindings/KindView';
import { tStored } from '../i18n/index.svelte';
import { call } from './invoke';

/** Every item that is not archived, oldest first. */
export function listItems(): Promise<ItemView[]> {
	return call<ItemView[]>('list_items');
}

/** Every kind an item can have, in the order they were made. */
export function listKinds(): Promise<KindView[]> {
	return call<KindView[]>('list_kinds');
}

/**
 * Saves an item. Rejects with a `CommandError` whose code says why it was refused (no name, a bad
 * quantity, an unknown kind). The views are told with `items_changed` once it is stored.
 */
export function addItem(draft: ItemDraftInput): Promise<ItemView> {
	return call<ItemView>('add_item', { draft });
}

/** The name of a kind in the user's language: a seed kind is translated, a typed name is shown as typed. */
export function kindName(kind: KindView): string {
	return kind.label.type === 'key' ? tStored(kind.label.value) : kind.label.value;
}
