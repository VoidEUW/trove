# Tier 3: the UI

Everything in `apps/desktop/src`. Tests live next to the code (`*.test.ts`) and in `src/test/`; the
browser tests are in `apps/desktop/e2e/`.

## Methods

| Method | Where | What it checks |
|---|---|---|
| Unit tests (Vitest) | `src/lib/api/*.test.ts`, `src/lib/i18n/i18n.test.ts` | The API wrappers (with a mocked `invoke`), error mapping, `t()` and locale matching. `fast-check` for rules that hold for any value |
| Component tests (Testing Library) | `src/lib/components/*.test.ts`, one file per component | Roles, `aria-*`, keyboard, pointer and click behaviour, snippets. `src/test/SidebarHarness.svelte` renders a sidebar with entries, because an entry reads its sidebar's state from context |
| Screen tests | `src/routes/items/page.test.ts` | The Items screen with the API mocked: list, empty and error states, refetch on `items_changed`, the add form |
| Accessibility, jsdom | `src/test/a11y.test.ts`, `src/test/axe.ts` | `axe-core` over each component and screen. No layout, so colour contrast is off here |
| Browser smoke | `e2e/items.spec.ts` | The built app in Chromium against `e2e/tauri-mock.js` (a stand-in for the Rust side): open, add, validate, sort, and `axe` including contrast in both themes |

## How to write them

- Components never call `invoke`; tests mock `$lib/api` (screens) or `@tauri-apps/api/core` (the API
  layer), and the CI guard still requires that only `src/lib/api/` imports `@tauri-apps/api`.
- Text comes from `en.json` (`en['items.add']`), not from copies in the test.
- A `beforeEach` that returns a mock calls it afterwards as cleanup: wrap the body in braces.
- The smoke tests set the theme through `data-theme` on `<html>` and turn on reduced motion, so an
  entry animation does not change colours mid-check.
- A known failure is written as `test.fail()` with the reason, so it goes red when fixed.

## Not covered

- jsdom has no layout and no focus trap for `<dialog>`, so dragging the sidebar edge is tested with
  synthetic pointer events (the clamped width, not real pixels) and Dialog's focus handling is the
  browser's own. A real browser would show both.
- Select and Menu are custom (a button and a menu), not the browser's `<select>`; their keyboard handling
  is tested. Menu has no type-ahead.
- The real window is checked by hand: WebDriver does not run against macOS webviews.
