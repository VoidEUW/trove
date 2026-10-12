# 15. The interface is in English and German from the first component

- Status: Accepted
- Date: 2026-10-12
- Origin: decided 2026-10-12
- Issue: #1

## Context

The app is used in German and English. Strings are cheap to route through translations while the
UI is small and expensive to retrofit later. Error text already lives in the UI
([4](0004-typed-command-boundary.md)); seed kinds ship with names ([6](0006-item-kinds-as-data.md)).

## Decision

- English is the source language; German is complete for every release.
- Every user-facing string goes through a message key, including error codes mapped from
  `AppError.code`. No literal UI text in components.
- Seed kinds and seed field labels store a message key; text a user types is stored as typed and
  never translated.
- Dates, numbers and money are formatted with `Intl` in the chosen locale. Lengths are stored in
  millimetres and shown in metric units.
- The language follows the system and can be changed in settings.
- Messages are the JSON files in `apps/desktop/src/lib/i18n/`, read through `t()`
  ([docs/languages.md](../languages.md)).

This rules out English-only strings in components, translated data in the database and a
second i18n library such as Paraglide JS.

## Consequences

- Every UI task includes both languages; a missing key fails the build.
- Follow-ups: `de.json`, key-coverage check in CI.
