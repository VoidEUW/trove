# 10. The operation log is the source of truth; SQLite is a rebuildable projection

- Status: Accepted
- Date: 2026-10-12
- Origin: raised 2026-10-12 ("op reconstruction of the inventory")
- Issue: #1

## Context

[9](0009-operation-log-and-sync.md) records every write as an operation, but
[8](0008-sqlite-storage.md) still makes `trove.db` the only live state. The goal is that the
inventory can always be reconstructed from operations, so a sync mistake or a corrupted database is
repaired by replaying. That is only true if replay is guaranteed to give the same state, which turns
the operation format into a long-lived contract. This must be settled before the first migration.

## Decision

- Operations are authoritative. The entity tables in `trove.db` are a projection: applying all
  operations from the latest snapshot in `(hlc, device_id)` order yields them.
- Every operation carries `op_version`. Old versions are read through **upcasters** in
  `trove-store` that turn them into the current version; an upcaster is never removed.
- Changing the projection schema means a migration *or* a rebuild by replay; both must give the
  same tables, and a test checks it.
- Snapshots at an HLC watermark bound replay time; a snapshot records the `op_version` it was made
  with.
- Data is never fixed with direct SQL; a fix is a new operation.

This rules out operations that cannot be replayed on their own (for example ones that rely on
derived data), and silent changes to the operation format.

## Consequences

- A broken projection or a sync error is repaired by replay; history, undo and the move log come
  from the same data.
- Every change to an operation type needs an upcaster and golden test fixtures for each old version.
- Start-up after a crash may need a replay from the last snapshot; snapshots keep it short.
- `trove.db` in [8](0008-sqlite-storage.md) is a projection, not the only live state.
- Follow-ups: operation type catalogue in core, replay function, projection-equals-replay test,
  golden fixtures per `op_version`.
