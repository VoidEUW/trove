# 16. Operations are JSON patches ordered by a packed 64-bit hybrid logical clock

- Status: Accepted
- Date: 2026-10-12
- Issue: #58

## Context

[9](0009-operation-log-and-sync.md) lists what an operation holds (operation id, device id, hybrid
logical clock (HLC), schema version, entity, field-level patch) but not how it is encoded.
[10](0010-operations-as-source-of-truth.md) makes the log the source of truth, so the format is a
long-lived contract: once operations exist, changing it needs a new `op_version`, an upcaster and
golden fixtures. It must be settled before the first migration (#5).

The encoding must order by `(hlc, device_id)`, export as one JSONL line per operation, replay without
derived data, and be defined in `trove-core` without IO. The first migration must not own entity
tables; items arrive with #9.

## Options

The patch is JSON in every option: it matches field values ([6](0006-item-kinds-as-data.md)), exports
to JSONL and can be queried with SQLite's JSON functions. CBOR or MessagePack save bytes but make the
log unreadable in `sqlite3` and in sync files. The options differ in the clock.

### A. Fixed-width text

`{wall_ms:013}-{counter:05}`, one `TEXT` column. String order is time order and the same text appears in
JSONL. Readable, but 18 bytes per row and a parse step for every comparison or increment.

### B. Structured columns

`wall_ms INTEGER, counter INTEGER`. No parsing, but every order and index is three columns wide and
JSON needs an object, so there are two representations.

### C. Packed integer

One 64-bit integer: 47 bits of Unix milliseconds and 16 bits of counter. It is compact and sorts
natively as one `INTEGER`. It is opaque when read raw, and its value (about 1.1 × 10¹⁷) exceeds 2⁵³,
so a JSON reader that uses floating-point numbers would round it.

## Decision

We chose **Option C** over the advised Option A. Reading the clock raw does not matter, because the
log is read by code (it can be decoded with `hlc >> 16`); compactness and native ordering do.

- **Patch**: a JSON object mapping field name to new value. `null` clears the field, so a field never
  holds a literal `null`. The first operation for an entity id creates the entity; archiving is the
  `archived` field. There is no delete operation ([5](0005-domain-model.md)).
- **Clock**: the 47 + 16 bits make a positive `i64` (about 4,400 years; 65,535 events per
  millisecond). SQLite stores it as `INTEGER`; JSON and JSONL carry it as a decimal string so no reader
  loses precision. Order is `ORDER BY hlc, device_id`.
- **Drift limit**: a constant in `trove-core`, 1 hour. A remote HLC further ahead of the local wall
  clock is refused. It is behaviour, not format, so changing it needs no `op_version`.
- **First migration**: `ops(op_id TEXT PRIMARY KEY, device_id TEXT, hlc INTEGER, op_version INTEGER,
  entity TEXT, entity_id TEXT, patch TEXT)` and a `meta` key/value table that holds the device id. No
  entity tables. The store applies an operation through a handler found by `entity`; tests register a
  test entity and each later issue registers its own.
- **Fixtures**: a golden fixture pins the JSON of every `op_version` from version 1.

This rules out binary patch encodings, a text clock and clock columns split in two.

## Consequences

- Ordering and replay use one indexed `INTEGER` column; clock arithmetic is plain integer maths in core.
- A raw `sqlite3` session shows an opaque clock. The integer in SQLite and the string in JSON must stay
  in step; one serde adapter and the golden fixture guarantee it.
- The 47 + 16 split is part of the format; changing it needs an `op_version`, an upcaster and fixtures.
- A device whose clock is more than an hour ahead has its operations refused; the UI will need a message
  for that case later.
- Follow-ups: #59 (types, clock, fixture), #60 (first migration), #61 (write path, replay).
