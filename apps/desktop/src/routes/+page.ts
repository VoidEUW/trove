import { redirect } from '@sveltejs/kit';

// The map is the home screen (ADR 14), but it does not exist yet: open the items instead.
export function load() {
	redirect(307, '/items');
}
