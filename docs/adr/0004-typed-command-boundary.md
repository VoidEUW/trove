# 4. The UI calls Rust only through typed Tauri commands and events

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "Crate map and the Rust ↔ Svelte boundary" in the first repository
- Issue: #1

## Context

Types and errors cross from Rust to TypeScript on every call. Hand-written TypeScript types drift.
`tauri-specta` types the commands themselves but tracks Tauri releases closely. Depends on
[3](0003-crate-map.md).

## Decision

- Structs and enums that cross the boundary derive `ts_rs::TS` in the crate that owns them and are
  exported to `apps/desktop/src/lib/bindings/`. The files are checked in; CI regenerates them and fails on a
  difference. The export runs only in tests, so `trove-core` stays free of IO.
- Each command has one thin hand-written wrapper in `apps/desktop/src/lib/api/`. Components never call
  `invoke` or use a command name directly.
- Commands live in `trove-desktop`, run store calls on a blocking thread and return
  `Result<T, AppError>`.
- `trove-core` and `trove-store` return their own `thiserror` types. Only `trove-desktop` maps them
  to `AppError { code, message }`; `code` is an exported enum and the UI maps it to text.
- After a committed write the shell emits `items_changed` or `layout_changed` with an exported
  payload. Views refetch on an event instead of patching their own state.

This rules out `tauri-specta`, hand-written TypeScript types and UI access to files or SQLite.

## Consequences

- Types cannot drift; a wrapper can still disagree with its command, so wrappers stay one call long.
- User-facing error text lives in the UI and can be translated ([15](0015-ui-languages.md)).
- Refetching on events is simple but costs extra reads; fine at household scale.
- Follow-ups: bindings export test, CI staleness check, first command end to end.
