# Tier 2: shell commands

The Tauri commands in `apps/desktop/src-tauri/src/` (`commands.rs`, `items.rs`), the mapping of errors
to `AppError` (`error.rs`) and the events (`events.rs`). The shell only translates between the UI and
the crates, so its tests check the translation, not the domain rules.

## Methods

- **Mock IPC.** `testing.rs` builds a mock Tauri app with a real store in a temporary folder
  (`app()`) and calls a command the way the UI does (`invoke()`). Each command has a test for its
  success value and for the error codes it returns (`name_required`, `quantity_invalid`,
  `unknown_kind`, `storage`).
- **Error mapping.** `error.rs` has one unit test per kind of crate error: `StoreError`, `ItemError`,
  `PatchError`, `IdError`, `ClockError`.
- **Events.** A test checks that `items_changed` is emitted after the write is committed, and that a
  failed write emits nothing.
- **Bindings.** `cargo test` regenerates the TypeScript types into `apps/desktop/src/lib/bindings`; CI
  fails when they differ from the checked-in files.

## Not covered

`lib.rs` (the startup that opens the data folder and the store) and `main.rs` are not run by tests: the
real window is checked by hand with `npm run tauri dev`.
