# 5. The domain is a normalised relational model of spaces, places and items

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "Data model" in the first repository
- Issue: #1

## Context

Items of very different kinds (devices, furniture with their screws and manuals, paintings, letters,
valuables, licence keys) must be placed in several homes, searched, counted, shared between people
and tracked for wear. Data from several devices must merge later ([9](0009-operation-log-and-sync.md)).

## Decision

One table per entity, a foreign key for every relation, all ids UUIDv7. Lengths are integer
millimetres.

- **Home** → **Floor** (level `i16`, name) → **Room** (name; width, depth, ceiling). One database
  holds several homes. How rooms are positioned is [7](0007-room-layout-solver.md).
- **Staircase**: lower room, offset in it, width, run, direction. The opening above is derived.
- **Place**: fixed furniture or a fixture in a room with geometry (x, y, w, d, h, rotation), or a
  compartment inside another place without geometry. An optional `item_id` makes the place itself
  an item (a bookshelf).
- **Item**: name, kind ([6](0006-item-kinds-as-data.md)), quantity (default 1; one row per kind of
  thing, not per physical unit), location, owners, condition, tags, field values, attachments,
  created/updated/archived. Location is exactly one of `InPlace { place, spot }`,
  `InRoom { room, x, y, z, spot }`, `InItem { container, spot }`, `Digital`, `Unplaced`.
- **Condition**: 1–5 with a note: Broken, Fragile, Worn, Good, Mint. 1 and 2 count as *needs
  attention*.
- **Containers**: an item whose kind is a container holds items through `InItem`. Boxes nest;
  cycles are refused. An item's effective place is found by walking up to the first container that
  is `InPlace` or `InRoom`, so moving a box moves its contents. A non-empty container cannot be
  archived.
- **Person**: name and initial. Items and people are many-to-many (ownership, sharing).
- **Link**: item ↔ item with a relation: part of, manual for, accessory of, other. A link never
  places an item.
- Items and kinds are archived, never deleted.

This rules out storing entities as JSON documents, one row per physical unit, and hard deletes.

## Consequences

- Search and the map get "where is it" from one walk up the container chain.
- Moving a container is one write, whatever it holds.
- Lifecycle states (lent, sold, given away) and purchase data are not modelled yet; adding them is
  a new ADR.
- Follow-ups: core types and validation, first migration, container-cycle tests.
