# Theme: Heaven

> Pale sky and night blue. Sun-gold for what you picked and the one thing to do next, a clear blue for anything found or live, airy serif headings over a quiet sans interface, soft rounded panels that float on the ground.

Schema `theme/1` · picture [`heaven.png`](heaven.png) · components and rules in [system.md](system.md)

## Looks

| Component | Look |
|---|---|
| `ui/sidebar` | float |
| `ui/sidebar-group` | open |
| `ui/sidebar-entry` | soft |
| `ui/topbar` | open |
| `ui/tabs` | soft |
| `ui/button` | soft |
| `ui/close-button` | soft |
| `ui/menu` | float |
| `ui/text-field` | open |
| `ui/search` | soft |
| `ui/select` | soft |
| `ui/toggles` | soft |
| `ui/card` | soft |
| `ui/collapsible` | soft |
| `ui/chips` | soft |
| `ui/data-table` | soft |
| `ui/tooltip` | soft |
| `ui/spinner` | open |
| `ui/progress-bar` | soft |
| `ui/skeleton` | soft |
| `ui/toast` | float |
| `ui/dialog` | float |
| `ui/state-page` | soft |

## Tokens

```css
:root {
	--font-display: 'Cormorant Garamond Variable', Georgia, 'Times New Roman', serif;
	--font-body: 'Newsreader Variable', Georgia, 'Times New Roman', serif;
	--font-ui: 'Albert Sans Variable', system-ui, -apple-system, 'Segoe UI', sans-serif;
	--font-mono: 'Fira Code Variable', ui-monospace, Menlo, Consolas, monospace;
	--ui-size: 14px;
	--body-size: 16px;
	--display-weight: 500;

	--radius-sm: 5px;
	--radius: 8px;
	--radius-lg: 16px;
	--radius-pill: 999px;
	--border-width: 1px;
	--focus-width: 2px;
	--space: 4.2px;

	--icon-size: 16px;
	--icon-stroke: 1.4;

	--ease: cubic-bezier(0.2, 0.7, 0.2, 1);
	--ease-emphasized: cubic-bezier(0.6, 0, 0.3, 1);
	--ease-spring: cubic-bezier(0.3, 1.35, 0.6, 1);

	--motion-sidebar-collapse: 242ms var(--ease); /* width */
	--motion-group-disclose: 176ms var(--ease); /* height-fade */
	--motion-entry-hover: 132ms var(--ease); /* tint */
	--motion-entry-active: 132ms var(--ease); /* bar */
	--motion-tabs-move: 242ms var(--ease-emphasized); /* slide */
	--motion-button-hover: 132ms var(--ease); /* lift */
	--motion-button-press: 88ms var(--ease); /* sink */
	--motion-menu-enter: 198ms var(--ease-spring); /* scale */
	--motion-field-focus: 132ms var(--ease); /* underline */
	--motion-search-focus: 0ms var(--ease); /* border */
	--motion-switch-toggle: 198ms var(--ease-emphasized); /* slide */
	--motion-collapse-disclose: 220ms var(--ease); /* height-fade */
	--motion-tooltip-enter: 330ms var(--ease); /* fade */
	--motion-spinner-loop: 1100ms linear; /* spin */
	--motion-bar-indeterminate: 1980ms var(--ease-emphasized); /* shimmer */
	--motion-skeleton-sweep: 1760ms linear; /* sweep */
	--motion-toast-enter: 330ms var(--ease); /* rise */
	--motion-modal-enter: 330ms var(--ease); /* zoom */
	--motion-page-enter: 198ms var(--ease); /* fade */
}

:root,
[data-theme='dark'] {
	--ground: #0f1824;
	--surface: #152132;
	--surface-raised: #1d2c41;
	--line: #2a3d56;
	--text: #eaf2fb;
	--text-muted: #a3b6cb;
	--text-faint: #71859b;
	--accent: #f2c64e;
	--focus: #6fb3f2;
	--success: #6cc79a;
	--warning: #f0a050;
	--danger: #f07a62;
	--hover: rgba(234, 242, 251, 0.06);
	--accent-soft: rgba(242, 198, 78, 0.16);
	--success-soft: rgba(108, 199, 154, 0.15);
	--warning-soft: rgba(240, 160, 80, 0.15);
	--danger-soft: rgba(240, 122, 98, 0.15);
	--on-accent: #0f1824;
	--on-danger: #0f1824;
	--scrim: rgba(0, 0, 0, 0.55);
	--shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
	--shadow-sm: 0 2px 8px rgba(0, 0, 0, 0.3);
	--skeleton-highlight: #2d3c50;
	color-scheme: dark;
}

[data-theme='light'] {
	--ground: #e5edf6;
	--surface: #d7e6f4;
	--surface-raised: #dde9f5;
	--line: #cbdbea;
	--text: #0f2338;
	--text-muted: #475e77;
	--text-faint: #66798f;
	--accent: #8f6400;
	--focus: #2574c4;
	--success: #1e7a50;
	--warning: #a85200;
	--danger: #c23c26;
	--hover: rgba(15, 35, 56, 0.05);
	--accent-soft: rgba(143, 100, 0, 0.12);
	--success-soft: rgba(30, 122, 80, 0.12);
	--warning-soft: rgba(168, 82, 0, 0.12);
	--danger-soft: rgba(194, 60, 38, 0.1);
	--on-accent: #ffffff;
	--on-danger: #ffffff;
	--scrim: rgba(15, 35, 56, 0.3);
	--shadow: 0 12px 32px rgba(15, 35, 56, 0.12);
	--shadow-sm: 0 2px 8px rgba(15, 35, 56, 0.08);
	--skeleton-highlight: #cdd9e6;
	color-scheme: light;
}
```

## Typography

| Role | Family | Package | Used for |
|---|---|---|---|
| Display | Cormorant Garamond | `@fontsource-variable/cormorant-garamond` | Wordmark, page and dialog titles, item titles on the detail page, empty states |
| Body | Newsreader | `@fontsource-variable/newsreader` | Item names in lists and panels, notes and long text |
| Interface | Albert Sans | `@fontsource-variable/albert-sans` | Every control, label and menu |
| Mono | Fira Code | `@fontsource-variable/fira-code` | Item IDs, measurements, licence keys, keyboard keys |

Interface text is 14px, long text 16px with a 1.6 line height and a 68ch measure. Headings use
weight 500. Uppercase labels (group and menu headings) are 10.5px, weight 600, letter-spacing 0.09em.

## Icons

- Library: [Lucide](https://lucide.dev/) (`@lucide/svelte`), outline.
- Stroke 1.4 on the 24-unit grid, drawn at 16px, `currentColor`.

## Motion

| Slot | Preset | Duration | Curve |
|---|---|---|---|
| `sidebar.collapse` | width | 242ms | standard |
| `group.disclose` | height-fade | 176ms | standard |
| `entry.hover` | tint | 132ms | standard |
| `entry.active` | bar | 132ms | standard |
| `tabs.move` | slide | 242ms | emphasized |
| `button.hover` | lift | 132ms | standard |
| `button.press` | sink | 88ms | standard |
| `menu.enter` | scale | 198ms | spring |
| `field.focus` | underline | 132ms | standard |
| `search.focus` | border | 0ms | standard |
| `switch.toggle` | slide | 198ms | emphasized |
| `collapse.disclose` | height-fade | 220ms | standard |
| `tooltip.enter` | fade | 330ms | standard |
| `spinner.loop` | spin | 1100ms | linear |
| `bar.indeterminate` | shimmer | 1980ms | emphasized |
| `skeleton.sweep` | sweep | 1760ms | linear |
| `toast.enter` | rise | 330ms | standard |
| `modal.enter` | zoom | 330ms | standard |
| `page.enter` | fade | 198ms | standard |

Tooltips show after a 250ms hover intent. Toasts sit top-centre and stay for 3s.

## Notes

- Derived from the Hera theme "Heaven" (`hera-theme/1`). Two light values were darkened to meet the
  contrast rules in system.md: `--text-faint` `#7a8fa5` → `#66798f` (was 2.8:1 on `--ground`) and
  `--accent` `#a87400` → `#8f6400` (text on a filled accent was 4.1:1). Every check now passes in
  both modes; the lowest are light `--text-faint` at 3.5:1 on `--surface` and light `--accent` at
  4.1:1 on `--surface`.
- Heaven had no success or warning colour. The ones added sit apart from `--accent` by hue in dark
  and by hue and lightness in light, but light `--warning` and `--accent` are both dark ambers:
  compare them in the rendered picture before applying.
- Heaven's rose "people and ownership" colour has no token in `theme/1`, so it is not in this file.
  Owner avatars use the neutral `ui/chips` avatar until that is decided (see the Trove idea export).
- `ui/sidebar-group` is `open` rather than Heaven's `float`, and `ui/dialog` and `ui/toast` are
  `float` rather than `open`: in `theme/1` an open dialog or toast would have no background.
