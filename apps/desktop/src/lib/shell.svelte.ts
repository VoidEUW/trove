// What the current screen puts in the app shell's top bar: its title and its toolbar.
import type { Snippet } from 'svelte';

class Shell {
	/** The title of the screen. Without one the top bar shows the app's name. */
	title = $state<string | undefined>();
	/** Buttons on the right of the top bar. */
	toolbar = $state<Snippet | undefined>();
}

export const shell = new Shell();
