// The accessibility check (ADR 17, tier 3): axe-core over rendered markup. jsdom has no layout, so the
// rules that need real pixels are off here: colour contrast comes from the theme tokens, and the
// page-level landmark rule does not apply to a single component.
import axe from 'axe-core';
import { expect } from 'vitest';

export async function expectAccessible(container: HTMLElement): Promise<void> {
	const { violations } = await axe.run(container, {
		rules: { 'color-contrast': { enabled: false }, region: { enabled: false } }
	});
	const report = violations
		.map((v) => `${v.id}: ${v.help} (${v.nodes.map((n) => n.target.join(' ')).join(', ')})`)
		.join('\n');
	expect(violations, report).toEqual([]);
}
