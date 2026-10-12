# 11. Search uses SQLite FTS5 and returns where each hit is

- Status: Proposed
- Date: 2026-10-11
- Origin: spec "Search and counting" in the first repository
- Issue: #1

## Context

Search is the main way in: a query lights up the furniture that holds a match on the map and shows
how many matches it contains. Counts per room, place, kind and owner appear in lists. Secrets must
never be searchable. SQLite with FTS5 is already bundled ([8](0008-sqlite-storage.md)).

## Decision

- One FTS5 table indexes item name, kind label, tags, owners, the location path
  (home › room › place › compartment) and the values of text fields. The store updates it in the
  same transaction as every write.
- A search returns item ids together with their effective place and room
  ([5](0005-domain-model.md)), which is what the map highlights.
- Counts are SQL aggregates that sum quantities.
- Secret fields and documents marked private are never indexed.

This rules out a second index such as `tantivy` and plain `LIKE` queries.

## Consequences

- One file to back up; the index cannot drift from the data because it is written in the same
  transaction.
- Prefix search, no fuzzy matching; German compound words need a tokenizer check.
- With [10](0010-operations-as-source-of-truth.md) the index is part of the projection and rebuilt
  with it.
- Follow-ups: FTS table and triggers or service hooks, search command, map highlight API.
