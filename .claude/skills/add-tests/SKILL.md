---
name: add-tests
description: Audit what the tests already cover for a described case, discuss the findings with the user, then write the agreed tests to the quality bar of ADR 17 and measure them.
argument-hint: <issue-number> "<what to test and where>"
disable-model-invocation: true
---

# Add tests: $ARGUMENTS

The first word of `$ARGUMENTS` is the issue number; the quoted text is what the user wants tested
and where, for example `61 "Test that replay still works when two operations share a millisecond"`.
Tests follow [ADR 17](docs/adr/0017-risk-tiered-quality-bar.md). The user decides what is worth
testing, so go **audit, discuss, write**, in that order. Do not write a test before step 4.

## 1. Read

- `gh issue view <issue>` and the code the text points at. If the text is vague about where, ask.
- `docs/adr/0017-risk-tiered-quality-bar.md`, `tools/quality/thresholds.json` and `docs/tests/` (it
  may not exist yet) for the tier notes and the fragile places already recorded.
- Split the text into separate cases, one per thing that could break. Decide the tier of the code:

| Tier | Code | Methods |
|---|---|---|
| 1 | `trove-core`, `trove-store`, migrations, operation log | unit and integration tests, `proptest` for rules that hold for any input, a golden fixture per `op_version`, mutation testing |
| 2 | shell commands | a mock-IPC test per command, including each error code |
| 3 | UI in `apps/desktop/src` | Vitest and Testing Library, an accessibility check, `fast-check` for pure functions |
| 4 | tools, docs | none required |

## 2. Audit

Search the existing tests for each case (`grep` the function, its inputs and its error paths in
`tests/`, in `#[cfg(test)]` modules and in `*.test.ts`). Report one row per case:

| Case | Verdict | Evidence |
|---|---|---|
| ... | **covered** / **partly covered** / **not covered** / **not relevant any more** | test name and file, or what is missing |

- *Covered* needs a test that would fail if the behaviour broke, not one that only runs the code.
- *Not relevant any more* means the code or the rule changed; say what changed.

For Tier 1 code, run the mutation tool on what the text points at and list every mutant no test
catches, with the line and the change:

```sh
cargo mutants -p <crate> --file <path> --re '<Type>::<function>'   # narrow to the code in question
cargo mutants -p <crate> --file <path>                              # the whole file when no function fits
```

For Tier 1 and 2, also run `cargo llvm-cov --workspace --json --output-path /tmp/cov.json` and
`node tools/quality/check.mjs coverage /tmp/cov.json`, and report the lines of that code left
uncovered.

Name the tests around it that are **redundant** (another test already asserts the same), **stale**
(they test removed behaviour) or **empty** (they run code but assert nothing). Do not delete any yet.

## 3. Discuss

Show the table, the surviving mutants and the weak tests, then propose what to add or remove, in
order of value. For each proposal say which method it uses and which finding it answers. Ask for
cases the user wants beyond these. **Stop and wait for the go-ahead.** Write only what they agree to.

## 4. Write

- A **bug** starts with a test that fails for the right reason. Run it and show the failure before
  you fix anything.
- New logic gets its test before or with the code.
- A rule that must hold for any input (ordering, round trips, validation limits, replay equals the
  live tables) gets a `proptest` property with bounded cases. Commit the inputs it finds, in the
  `proptest-regressions` file.
- A data format gets a golden fixture per `op_version` in `tests/fixtures/`. Never edit an existing
  fixture to make a test pass: that is a format change and needs an upcaster and an ADR.
- Each surviving mutant gets a test that kills it. If none can (the change makes no observable
  difference), add the mutant to `.cargo/mutants.toml` `exclude_re` with a comment giving the reason.
- A command test goes through mock IPC (`src-tauri/src/testing.rs`) and covers each error code.
- Store tests use a temporary directory (`tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))`) and
  never real data.
- Remove or fix the redundant, stale or empty tests that were agreed.
- Name a test after the behaviour it protects, not the function it calls.

## 5. Measure

Run what CI runs (the commands of `/open-pr` step 2), then the quality bar for the tier:

```sh
cargo llvm-cov --workspace --json --output-path /tmp/cov.json
node tools/quality/check.mjs coverage /tmp/cov.json
cargo mutants -p <crate> --file <path>          # Tier 1 only
node tools/quality/check.mjs mutation <crate> mutants.out/outcomes.json
```

Show the before and after numbers, and the mutants that still survive. If a number is below its
threshold, say so; do not lower `tools/quality/thresholds.json` (the ratchet forbids it). If it is
well above, offer to raise it.

## 6. Record

If the work added or removed a fragile place, update its note under `docs/tests/`: what can break,
the test that guards it, and what is not covered. Create the note if `docs/tests/` has none yet, as
reference notes only, never decisions or plans. Then list for the user the **Try to break this**
items: the fragile spots, the test for each, and what is still not covered. `/open-pr` puts them in
the pull request body.

Do not commit. The user runs `/open-pr` when they are satisfied.
