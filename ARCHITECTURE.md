# Architecture

How Trove's parts fit together **now**. Why each part looks the way it does is in the ADRs a
section cites; what is planned is in the issues. Little application code exists yet, so this file
describes the structure every issue builds towards.

Rules for this file:

- Every section ends with a *Source* line naming its ADRs. A section that rests on a *proposed* ADR
  says so in its heading.
- Describe the current state in the present tense. Plans and progress belong in issues.
- Change this file in the pull request that changes the structure or accepts an ADR.

## 1. Shape

```
┌──────────────────────────── Trove desktop app (Tauri 2) ─────────────────────────────┐
│  apps/desktop/           SvelteKit SPA, Svelte 5, Threlte                            │
│                          screens · map scene · layout editor · tokens · i18n         │
│                                │  typed commands and events                          │
│  apps/desktop/src-tauri  Tauri shell (trove-desktop): commands, events, data folder  │
│                                │                                                     │
│  crates/trove-core       domain types, validation, layout solver, services (no IO)   │
│                                │  ports (traits) implemented by                      │
│  crates/trove-store      SQLite, migrations, operation log, files, search index      │
└──────────────────────────────────────────────────────────────────────────────────────┘
                                 │
                   data folder (~/.trove or %APPDATA%\trove)
                                 │  later: operation files only
                   sync folder (e.g. a shared iCloud Drive folder)
```

Trove is one local process. The Rust core runs inside the Tauri app; there is no server, no
account and no network dependency. Dependencies point downwards only.

*Source:* [ADR 2](docs/adr/0002-desktop-app-with-tauri-and-svelte.md),
[ADR 3](docs/adr/0003-crate-map.md)

## 2. Parts

| Part | Path | Responsibility | Depends on | Must not |
|---|---|---|---|---|
| Core | `crates/trove-core` | Domain types, validation, layout solver, services, ports | — | Do IO of any kind |
| Store | `crates/trove-store` | SQLite, migrations, operation log, attachments, search index | core | Guess the data folder |
| Desktop shell | `apps/desktop/src-tauri` (`trove-desktop`) | Tauri commands and events, error mapping, data-folder resolution | core, store | Contain domain logic |
| UI | `apps/desktop/` | Screens, map scene, layout editor, components, translations | shell's commands | Read files or the database |
| Decisions | `docs/adr/` | One file per decision | — | Hold plans |
| Test notes | `docs/tests/` | How each tier is tested and measured, and the fragile places | — | Hold decisions or plans |
| Tools | `tools/` | `quality` (coverage, mutation and ratchet checks), `theme-check` (token block and literal values), `theme-preview` (renders a theme's picture) | — | Ship in the app |
| Component system | `docs/themes/system.md` | Token names, looks, motion presets, component catalogue | — | Differ between themes |
| Theme | `docs/themes/heaven.md`, `heaven.png` | Spec (looks, tokens, type, icons, motion) and its rendered picture | system | Be edited in `tokens.css`; that file is generated from the spec |

*Source:* [ADR 1](docs/adr/0001-record-decisions-as-adr-files.md),
[ADR 3](docs/adr/0003-crate-map.md), [ADR 14](docs/adr/0014-ui-architecture-and-tokens.md)

## 3. Boundaries

- **UI ↔ Rust**: only Tauri commands and events. Boundary types derive `ts_rs::TS` and are exported
  to `apps/desktop/src/lib/bindings/` (checked in; CI fails when stale). Each command has one thin
  wrapper in `apps/desktop/src/lib/api/`; components never call `invoke`.
- **Errors**: core and store return their own `thiserror` types; only the shell maps them to
  `AppError { code, message }`. The UI shows text for `code`, in the user's language.
- **Events**: `items_changed` and `layout_changed` after a committed write; views refetch.
- **Core ↔ store**: core defines ports (traits); store implements them; the shell wires them.

*Source:* [ADR 3](docs/adr/0003-crate-map.md), [ADR 4](docs/adr/0004-typed-command-boundary.md)

## 4. Data

| Entry in the data folder | Contents |
|---|---|
| `trove.db` | SQLite in WAL mode: `ops`, entity tables projected from it, FTS index |
| `files/` | Attachments by SHA-256 (`files/ab/cdef…`) |
| `backups/` | Snapshots taken with the SQLite backup API |
| `config.toml` | Settings that are not secrets |
| `logs/` | Rotating log files |

- Location: `~/.trove` (macOS, Linux), `%APPDATA%\trove` (Windows), overridden by `TROVE_HOME`. The
  shell resolves it and hands the path to the store, which never reads `TROVE_HOME`; tests pass a
  temporary folder.
- `rusqlite` with bundled SQLite, synchronous; one writer connection, read connections as needed,
  foreign keys on. Commands call the store on a blocking thread.
- The operation log is authoritative. The entity tables are a projection: replaying all operations
  from the latest snapshot in `(hlc, device_id)` order rebuilds them. Data is never fixed with
  direct SQL; a fix is a new operation.
- Migrations: `rusqlite_migration`, embedded, forward-only, `user_version`, run on open. A database
  newer than the app is refused. A migration and a rebuild by replay must give the same tables.
- Every operation carries `op_version`; old versions are read through upcasters in the store, which
  are never removed and are covered by golden fixtures.
- The live database never lives in a synced folder.
- *Proposed in ADR 12*: secrets are encrypted per value with a data key held in the OS keychain.

*Source:* [ADR 8](docs/adr/0008-sqlite-storage.md),
[ADR 10](docs/adr/0010-operations-as-source-of-truth.md),
[ADR 12](docs/adr/0012-attachments-and-secrets.md) (proposed)

## 5. Domain

- **Home → Floor → Room**; rooms are `Free` or `Attached` to another room's wall. The **layout
  solver** in core is the only producer of absolute positions; staircase openings are derived.
- **Place**: furniture with geometry in a room, or a compartment in another place; may itself be an
  item.
- **Item**: name, kind, quantity, location (`InPlace`, `InRoom`, `InItem`, `Digital`, `Unplaced`),
  owners, condition 1–5 (Broken … Mint), tags, field values, attachments; archived, never deleted.
- **Containers** nest without cycles; an item's effective place is found by walking up its
  containers.
- **Kind**: data, not code: icon, counting (`Single` / `Counted`), container flag, typed field
  definitions. Field values are JSON validated by core on every write. Seed kinds have fixed ids.
- **Person** (many-to-many with items) and **Link** (part of, manual for, accessory of, other).
- Ids are UUIDv7; lengths are integer millimetres.

*Source:* [ADR 5](docs/adr/0005-domain-model.md), [ADR 6](docs/adr/0006-item-kinds-as-data.md),
[ADR 7](docs/adr/0007-room-layout-solver.md)

## 6. Change, history and sync

- Every mutation writes an operation (id, device, HLC, `op_version`, entity, field patch) to `ops`
  in the same transaction. Only the store's service layer writes to SQLite. History, undo and the
  move log come from the same operations.
- Encoding: the patch is a JSON object of field to value, with `null` meaning clear; the first
  operation for an entity id creates it. The HLC is a packed `i64` (47 bits of Unix milliseconds,
  16 bits of counter), an `INTEGER` in SQLite and a decimal string in JSON. Order is
  `(hlc, device_id)`; a remote HLC more than 1 hour ahead of the local clock is refused. The first
  migration creates `ops` and `meta` (device id) only; the store applies operations through a handler
  per entity.
- Projections: each entity type has a `Projection` handler in `trove-store` that applies its operations
  inside the transaction that appends them, and empties its tables for a replay. `kind` and `item` are
  the first two. The eleven seed kinds have fixed ids and are inserted by a migration; a replay puts
  them back. Validation lives in `trove-core` and runs before anything is written: a name needs a
  visible character (whitespace, control and zero-width characters alone are refused) and a quantity
  is a whole number from 1.
- Snapshots at an HLC watermark bound replay time and record the `op_version` they were made with.
- Sync (not built yet): each device appends its operations to
  `<sync folder>/ops/<device_id>/<seq>.jsonl`; others import them idempotently; per field the highest
  `(hlc, device_id)` wins; container cycles from merges are repaired deterministically; snapshots
  bound the log. A broken projection or a sync mistake is repaired by replay. Attachments sync by
  hash.

*Source:* [ADR 9](docs/adr/0009-operation-log-and-sync.md),
[ADR 10](docs/adr/0010-operations-as-source-of-truth.md),
[ADR 16](docs/adr/0016-operation-encoding-and-clock-format.md)

## 7. User interface

- SvelteKit with `adapter-static`, no SSR. Routes: `/` map, `/items`, `/items/[id]`,
  `/layout/[home]`, `/settings`. State in `.svelte.ts` modules per domain with runes.
- Components follow the catalogue in `docs/themes/system.md` and live in
  `apps/desktop/src/lib/components/`, named after their id. Built so far: Button, Menu, TextField,
  Select, Dialog, StatePage, Sidebar, SidebarEntry, Topbar and DataTable.
- Built so far: the app shell (`routes/+layout.svelte`: sidebar, top bar, the screen's title and toolbar
  set through `lib/shell.svelte.ts`) and the Items screen. `/` redirects to `/items` until the map exists.
  Screens call `lib/api/` only: one wrapper per command, `CommandError` with a code that maps to a
  message key in `en.json`, and listeners for `items_changed` and `layout_changed`.
- Look: the Heaven theme as specified in `docs/themes/heaven.md`; `/apply-theme` generates
  `apps/desktop/src/lib/theme/tokens.css` from it. No literal colours, fonts, radii or durations in
  `apps/desktop/src`. `--accent` = selection, `--focus` = found and live, `--warning` = fragile,
  `--danger` = needs attention.
- Map: one Threlte scene drawn from solver output; orthographic camera for 2D and the layout
  editor, perspective camera for 3D; raycast picking; scene colours read from the tokens.
- Languages: English (source) and German; every string is a key in
  `apps/desktop/src/lib/i18n/*.json` used with `t()`; seed kinds store keys, user text is stored as
  typed.

*Source:* [ADR 13](docs/adr/0013-threlte-floor-plan-scene.md),
[ADR 14](docs/adr/0014-ui-architecture-and-tokens.md),
[ADR 15](docs/adr/0015-ui-languages.md)

## 8. Cross-cutting

- **Errors**: typed per crate, mapped once in the shell; no `unwrap` outside tests, `expect` only
  at startup in the shell.
- **Ids**: UUIDv7 everywhere, generated in core.
- **Search** *(proposed in ADR 11)*: one FTS5 table written in the same transaction as the data;
  results carry their effective place and room; secrets and private documents are never indexed.
- **Secrets**: never logged, never indexed, revealed only on demand.
- **Testing**: a risk-tiered bar. The data path (core, store, migrations, operation log) needs line
  coverage ≥ 90%, mutation score ≥ 85%, property tests and a golden fixture per `op_version`; shell
  commands need ≥ 80% and IPC tests; the UI gets component, accessibility and browser smoke tests. CI
  holds the numbers, and every pull request lists "Try to break this" cases. Core is pure, store tests use
  a temporary folder and replay is tested against the migrated projection. How each tier is tested,
  how to measure it and where it is fragile is in [`docs/tests/`](docs/tests/README.md).

*Source:* [ADR 4](docs/adr/0004-typed-command-boundary.md),
[ADR 5](docs/adr/0005-domain-model.md), [ADR 10](docs/adr/0010-operations-as-source-of-truth.md),
[ADR 11](docs/adr/0011-search-with-fts5.md) (proposed),
[ADR 12](docs/adr/0012-attachments-and-secrets.md) (proposed),
[ADR 17](docs/adr/0017-risk-tiered-quality-bar.md)

## 9. Decisions

| ADR | Decision | Status |
|---|---|---|
| [1](docs/adr/0001-record-decisions-as-adr-files.md) | Decisions are ADR files; plans live only in issues | accepted |
| [2](docs/adr/0002-desktop-app-with-tauri-and-svelte.md) | Local-first Tauri 2 app with a Svelte 5 UI | accepted |
| [3](docs/adr/0003-crate-map.md) | Three crates, dependencies point one way | accepted |
| [4](docs/adr/0004-typed-command-boundary.md) | Typed commands and events between UI and Rust | accepted |
| [5](docs/adr/0005-domain-model.md) | Normalised relational domain model | accepted |
| [6](docs/adr/0006-item-kinds-as-data.md) | Item kinds are data with typed fields | accepted |
| [7](docs/adr/0007-room-layout-solver.md) | Rooms attach to walls; one solver places them | accepted |
| [8](docs/adr/0008-sqlite-storage.md) | One SQLite database in a local data folder | accepted |
| [9](docs/adr/0009-operation-log-and-sync.md) | Field-level operation log; sync by operation files | accepted |
| [10](docs/adr/0010-operations-as-source-of-truth.md) | Operations are the source of truth | accepted |
| [11](docs/adr/0011-search-with-fts5.md) | Search with SQLite FTS5 | proposed |
| [12](docs/adr/0012-attachments-and-secrets.md) | Attachments by hash; secrets with a keychain key | proposed |
| [13](docs/adr/0013-threlte-floor-plan-scene.md) | One Threlte scene for 2D and 3D | accepted |
| [14](docs/adr/0014-ui-architecture-and-tokens.md) | SvelteKit SPA styled only by Heaven tokens | accepted |
| [15](docs/adr/0015-ui-languages.md) | English and German from the first component | accepted |
| [16](docs/adr/0016-operation-encoding-and-clock-format.md) | JSON patch operations ordered by a packed 64-bit HLC | accepted |
| [17](docs/adr/0017-risk-tiered-quality-bar.md) | Risk-tiered tests: mutation and property tests on the data path | accepted |

## 10. Open questions

| Question | Blocks | Settled by |
|---|---|---|
| Do ADRs 1–15 hold against the MVP prototype? | Every milestone after Foundation | Prototype review; ADRs that fail are removed or superseded |
| Do people and ownership get their own colour? | Owners and the Shared mark | `adr` issue from the prototype import |
| Small text in `--text-faint` (table header labels, short ids) is 3.7–4.3:1, below the 4.5:1 of WCAG AA. Darker token or another colour for that text? | Contrast test in `e2e/items.spec.ts` | Theme change in `docs/themes/heaven.md` (#100) |
| How does a second device get the secrets' data key (ADR 12)? | Sync of licence keys | Spec issue, before sync |
| Walls fixed low or cut away by camera angle; HTML labels or SDF text (ADR 13)? | Map scene | Spike in the 3D map phase |
| Lifecycle states (lent, sold, given away) and purchase data | Domain model extension | New ADR when the feature is planned |
