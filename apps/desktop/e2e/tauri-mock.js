// A stand-in for the Rust side (ADR 17, tier 3). The smoke tests run the built UI in a browser, where
// there is no Tauri, so this answers the commands the way the shell does and emits `items_changed`.
// Items live in sessionStorage, so they survive a reload within one test. `window.__mock.calls` lists the
// commands called, and `window.__mock.failNext` makes the next call reject with the given AppError.
(() => {
	let counter = 0;
	const callbacks = {};
	const listeners = {};
	const seedKeys = [
		'electrical', 'furniture', 'artwork', 'documents', 'memories', 'valuables',
		'software_key', 'parts_and_manuals', 'tools', 'other', 'box'
	];
	const kinds = seedKeys.map((key, index) => ({
		id: `00000000-0000-7000-8000-${String(index + 1).padStart(12, '0')}`,
		label: { type: 'key', value: `kind.${key}` },
		counting: key === 'documents' || key === 'memories' || key === 'parts_and_manuals' || key === 'other' ? 'counted' : 'single',
		container: key === 'box'
	}));
	let items = JSON.parse(sessionStorage.getItem('mock-items') || '[]');
	window.__mock = { calls: [], failNext: null };

	const emit = (event, payload) =>
		(listeners[event] || []).forEach((id) => callbacks[id]?.({ event, id, payload }));

	window.__TAURI_INTERNALS__ = {
		transformCallback(callback) {
			const id = ++counter;
			callbacks[id] = callback;
			return id;
		},
		unregisterCallback(id) {
			delete callbacks[id];
		},
		convertFileSrc: (path) => path,
		async invoke(command, args) {
			window.__mock.calls.push(command);
			if (window.__mock.failNext) {
				const error = window.__mock.failNext;
				window.__mock.failNext = null;
				throw error;
			}
			if (command === 'plugin:event|listen') {
				(listeners[args.event] ||= []).push(args.handler);
				return args.handler;
			}
			if (command === 'plugin:event|unlisten') return;
			if (command === 'list_kinds') return kinds;
			if (command === 'list_items') return items;
			if (command === 'add_item') {
				const draft = args.draft;
				if (!draft.name.trim()) throw { code: 'name_required', message: 'an item needs a name' };
				if (!Number.isInteger(draft.quantity) || draft.quantity < 1)
					throw { code: 'quantity_invalid', message: 'bad quantity' };
				const kind = kinds.find((candidate) => candidate.id === draft.kind);
				if (!kind) throw { code: 'unknown_kind', message: 'no such kind' };
				const item = {
					id: `0199c3a0-0000-7000-8000-${String(++counter).padStart(12, '0')}`,
					name: draft.name.trim(),
					kind,
					quantity: draft.quantity,
					created_ms: Date.now()
				};
				items = [...items, item];
				sessionStorage.setItem('mock-items', JSON.stringify(items));
				setTimeout(() => emit('items_changed', { ids: [item.id] }), 0);
				return item;
			}
			throw { code: 'internal', message: `unknown command ${command}` };
		}
	};
	window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
})();
