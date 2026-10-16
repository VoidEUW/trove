// Events that say a write was committed (ADR 4). A view refetches when one arrives.
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { ItemsChanged } from '../bindings/ItemsChanged';
import type { LayoutChanged } from '../bindings/LayoutChanged';

/** Calls `handler` after items were written. Call the returned function to stop listening. */
export function onItemsChanged(handler: (payload: ItemsChanged) => void): Promise<UnlistenFn> {
	return listen<ItemsChanged>('items_changed', (event) => handler(event.payload));
}

/** Calls `handler` after homes, floors or rooms were written. Call the returned function to stop. */
export function onLayoutChanged(handler: (payload: LayoutChanged) => void): Promise<UnlistenFn> {
	return listen<LayoutChanged>('layout_changed', (event) => handler(event.payload));
}
