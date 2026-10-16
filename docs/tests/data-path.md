# Tier 1: the data path

`trove-core` (types, validation, clock, operations), `trove-store` (SQLite, migrations, operation log,
projections) and the migrations. A bug here can corrupt history that cannot be rebuilt, so it gets
the most methods.

## Methods

| Method | Where | What it checks |
|---|---|---|
| Unit tests | `#[cfg(test)]` modules in `crates/trove-core/src/` | One rule each, with the edges listed. Example: `ItemDraft::validate`, `Hlc` packing |
| Property tests (`proptest`) | `crates/trove-core/tests/properties.rs`, `crates/trove-store/tests/replay_properties.rs` | A rule for any input: clock ordering and `tick`/`receive`, patch last-write-wins, operation round trips, validation limits, replay equals the live tables |
| Integration tests | `crates/trove-store/tests/` (`open.rs`, `ops.rs`, `items.rs`, `projections.rs`) | The real schema: open, migrate, back up, write, replay, and the item and kind handlers |
| Golden fixtures | `crates/trove-core/tests/fixtures/op_v1.json`, `op_v1_kind.json` | The JSON of an operation, pinned for `op_version` 1 |
| Mutation testing | `cargo mutants` | That the tests above notice a change to the code |

## How to write them

- Store tests use a temporary folder under Cargo's scratch directory
  (`tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))`, helper `common::data_folder()`) and never read
  `TROVE_HOME` or real data.
- `tests/ops.rs` uses a made-up entity (`thing`) with its own table, so the log can be tested without
  the real handlers.
- Property tests keep a bounded number of cases (48 to 256) and write failing inputs next to the test file
  (`*.proptest-regressions`). Commit those files.
- A fixture is never edited to make a test pass. A change of format needs a new `op_version`, an
  upcaster in the store and a new fixture (CLAUDE.md, ADR 16).
- A bug fix starts with a test that fails for the right reason.
