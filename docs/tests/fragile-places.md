# Fragile places

Where a change is most likely to break something that matters, the test that guards it, and what is
not covered. Add a row when `/add-tests` or a review finds a new one; remove it when the place goes away.

## Data path

| Place | What can break | Guarded by | Not covered |
|---|---|---|---|
| Replay order (`oplog::ORDERED`, `Store::replay`) | The projection differs from the live tables, or depends on the order operations arrived in | `replay_properties.rs`: random histories, log copied in shuffled order, in place and into an empty database. Checked once by making replay `ORDER BY op_id`: it fails | Histories over 29 steps, many devices, a large log |
| Hybrid logical clock (`hlc.rs`) | Time goes backwards, counter wraps, packing exceeds `i64`, a far-future remote clock is accepted | `properties.rs` (`tick`, `receive`, packing, order) and unit tests at the edges (`MAX_DRIFT_MS` is exactly one hour, `0` is valid, `i64::MAX`) | The store does not call `receive` for operations from another device yet, so that path has no test (sync issue) |
| Operation JSON (`op.rs`) | A format change silently rewrites history | Golden fixtures `op_v1.json` (item) and `op_v1_kind.json` (kind); round-trip property | There is only `op_version` 1, so no upcaster is tested. Both fixtures are hand-built |
| Patch (`Patch::set`, `clear`) | A literal `null` is stored, an unnamed field is accepted, a later write is lost | `properties.rs`: last write wins, no `null`, no empty field; unit tests | Very large patches |
| Item validation (`ItemDraft::validate`) | A zero, fractional, huge or non-finite quantity gets in; a name is not trimmed, or a name with nothing visible in it (whitespace, control or zero-width characters) is accepted | Unit tests on the edges, and properties over every `f64` and over names built from whitespace, invisible, control and visible characters | Other characters that render as blank (for example some symbols in unusual fonts); directional marks are deliberately accepted |
| Kind and item handlers (`trove-store/src/items.rs`) | An operation is half applied, a wrong type is accepted, a seed kind is not restored by replay | `projections.rs`: create, change, archive, each refusal leaves ops and tables unchanged; `items.rs` replay test | Concurrent writers |
| Open and migrate (`store.rs`, `backup.rs`) | A newer database is touched, an upgrade is not backed up, the device id changes | `open.rs`: a newer database is refused and left byte for byte; one backup per upgrade, none when up to date; stable device id | A migration that fails halfway (the SQL is transactional; not exercised) |
| `Store::reader`, `configure` | Connections lose foreign keys or the busy timeout | Nothing can: the bundled SQLite and rusqlite already default to the same values, so the mutants are skipped in `.cargo/mutants.toml` | A dependency changing those defaults |

## Shell

| Place | What can break | Guarded by | Not covered |
|---|---|---|---|
| Error mapping (`error.rs`) | A crate error reaches the UI with the wrong code | One unit test per kind of crate error | New error variants must be added by hand; the compiler only checks that the match is complete |
| `items_changed` (`items.rs`) | The event fires before the commit or after a failure | `items_changed_is_emitted_after_the_item_is_committed`, `a_failed_save_emits_no_event` | `layout_changed` has no command yet |
| TypeScript bindings | The UI and Rust disagree about a type | `cargo test` regenerates them and CI fails on a difference | |
| Startup (`lib.rs`) | The data folder or store cannot be opened | Checked by hand with `npm run tauri dev` | No automated test |

## UI

| Place | What can break | Guarded by | Not covered |
|---|---|---|---|
| `toCommandError` (`errors.ts`) | A thrown value of any shape crashes error handling. It threw for an object with no prototype | Property test over `fc.anything({ withNullPrototype: true })` and an explicit case | |
| Locale matching (`i18n/index.svelte.ts`) | `de-AT` stops falling back to `de`, a missing key shows nothing | `i18n.test.ts` | Only `en` exists; `de` is injected by the test (issue #10) |
| `ui/data-table` | Sorting is reported wrongly, a click in the actions cell selects the row, a key on an inner button opens the row | `DataTable.test.ts` | Hover vs selected tint in the light theme is a design question, not tested |
| Add-item form | Invalid input reaches the shell, a refusal is shown on the wrong field | `page.test.ts` and `e2e/items.spec.ts` | |
| Colour contrast of small text | Header labels and short ids use `--text-faint`: 4.26:1 and 3.7:1 against 4.5:1 | `e2e/items.spec.ts` marks it `test.fail()`, so it goes red once fixed | Decision on the token or the table is open (#100) |
| Stand-in for the shell (`e2e/tauri-mock.js`) | It drifts from the real commands, so the smoke tests pass against behaviour that no longer exists | Nothing | A contract test between the mock and the shell (#101) |
| Menu, Select, Sidebar and the other components (`src/lib/components`) | Keyboard handling and focus (arrow keys wrap, Escape and Tab close), a click outside closes a menu but not on its opener, a submenu opens and returns focus, the sidebar width leaves its limits, a loading button still fires | One test file per component; Sidebar width also by a property over any sequence of arrow keys. Thirteen deliberate breaks in these components each failed at least one test | Real pixels and focus trapping (jsdom has neither), and Menu type-ahead (not implemented) |
