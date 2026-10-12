# 9. Every write records a field-level operation; devices sync by exchanging operation files

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "Change log and sync" in the first repository
- Issue: #1

## Context

Trove starts on one Mac, but sync through a folder such as iCloud Drive must be possible later
without reworking the data. The database file itself cannot be synced
([8](0008-sqlite-storage.md)). Adding a change log after data exists means data without history.

## Decision

- Every mutation writes, in the same transaction, an operation record to `ops`: operation id,
  device id, hybrid logical clock (HLC), schema version, entity, field-level patch. Only the store's
  service layer writes it; nothing writes to SQLite from elsewhere.
- **Export**: each device appends its own operations to `<sync folder>/ops/<device_id>/<seq>.jsonl`.
  One writer per directory; segments are written to a temporary name and renamed, and a closed
  segment never changes.
- **Import**: other devices' operations are applied by operation id, so importing twice changes
  nothing. A truncated last line or a segment not yet downloaded is retried later. An operation with
  a newer schema version than the app is refused.
- **Merge**: per field, the highest `(hlc, device_id)` wins. A location is one atomic field. A merge
  that would form a container cycle is repaired deterministically: the later mover becomes
  `Unplaced`. The clock refuses timestamps beyond a drift limit. Seed rows have fixed ids.
- **Size**: snapshots at an HLC watermark bound the log; segments are removed only after every
  known device has acknowledged them, and a new device starts from a snapshot. Attachments sync
  separately by hash.

This rules out syncing `trove.db`, a sync server, and CloudKit for now (the operation is the unit,
so another transport can be added later).

## Consequences

- Until sync exists the log costs one extra row per write and already gives history.
- Last-writer-wins per field can lose a concurrent edit to the same field; acceptable for a
  household.
- Sharing between two Apple IDs needs a shared iCloud folder, not plain iCloud Drive.
- Follow-ups: `ops` table in the first migration, HLC, export/import, snapshotting.
