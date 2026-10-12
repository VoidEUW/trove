# 3. The Rust side is three crates whose dependencies point one way

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "Crate map and the Rust ↔ Svelte boundary" in the first repository
- Issue: #1

## Context

The app embeds its core ([2](0002-desktop-app-with-tauri-and-svelte.md)). Domain logic such as
validation and the layout solver must be testable without disk or Tauri, and must be reusable by a
later phone app.

## Decision

| Part | Holds | Depends on |
|---|---|---|
| `crates/trove-core` | Domain types, validation, layout solver, services, ports (traits). No IO. | nothing internal |
| `crates/trove-store` | SQLite, migrations, operation log, files, search index; implements the ports | `trove-core` |
| `apps/desktop/src-tauri` (crate `trove-desktop`) | Tauri shell: commands, events, data-folder resolution, window, menu | `trove-core`, `trove-store` |
| `apps/desktop/` | SvelteKit SPA: screens, 3D scene, layout editor, tokens | `trove-desktop`'s commands only |

This rules out a server process, IO inside `trove-core`, and a UI that reads files or the database.

## Consequences

- Core logic is tested with plain unit and property tests.
- Ports add trait boilerplate between core and store.
- A phone app reuses `trove-core` and `trove-store` behind its own shell.
- Follow-ups: rename `crates/core` to `crates/trove-core` and the shell crate `app` to
  `trove-desktop`, add `crates/trove-store`, remove the template's `greet` example.
