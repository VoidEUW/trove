# 6. Item kinds are user-editable data with typed field definitions

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "Item kinds and fields" in the first repository
- Issue: #1

## Context

Each kind of thing needs different details: a device has a serial number and warranty, furniture
has assembly parts and a manual, a licence has a secret key and no location. New kinds will keep
appearing, and a release per new kind is not acceptable. Depends on [5](0005-domain-model.md).

## Decision

- A **Kind** is a row: name, icon, counting, a `container` flag and an ordered list of field
  definitions. Counting is `Single` (quantity always 1) or `Counted { unit }` (1 or more, optional
  unit label).
- A **field definition** has a stable id, label, type, `required` and type options. Types: text,
  long text, number, date, money, switch, item link, attachment, secret.
- **Values** are one JSON object in `items.fields`, keyed by field id. `trove-core` validates it
  against the kind on every write: unknown ids, wrong types and missing required fields are typed
  errors. Money is `{ amount_minor, currency }`, dates are ISO 8601, an item link must point to an
  existing item. Attachment and secret values hold only a reference
  ([12](0012-attachments-and-secrets.md)).
- Removing a field hides it and keeps its values. Changing the type of a field that has values is
  refused. Kinds are archived, never deleted.
- The mockup's kinds ship as seed rows with fixed ids: Electrical, Furniture, Artwork, Documents,
  Memories, Valuables, Software key, Parts & manuals, Tools, Other, plus Box (container). They are
  edited like any other kind.

This rules out kinds as Rust enums, one table per kind and untyped free-form fields.

## Consequences

- New kinds and fields need no code or migration.
- Field values are not SQL columns: filtering by a field needs JSON functions or the search index.
- Seed rows need translatable names ([15](0015-ui-languages.md)) and fixed ids so devices agree.
- Follow-ups: kind and field types in core, validator, kind editor UI, seed migration.
