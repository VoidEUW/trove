# 17. Tests are risk-tiered: mutation and property tests on the data path, lighter further out

- Status: Accepted
- Date: 2026-10-14
- Issue: #88

## Context

Trove keeps a household's records for years. The operation log is the source of truth and migrations are
forward-only ([8](0008-sqlite-storage.md), [9](0009-operation-log-and-sync.md),
[10](0010-operations-as-source-of-truth.md)), so a data bug can corrupt history that cannot be rebuilt. A
rendering bug is cheap by comparison. Today the Rust crates have unit and integration tests, the frontend has
none ([14](0014-ui-architecture-and-tokens.md) names Vitest and Testing Library, but they are not installed),
and nothing measures how much the tests cover or how well they catch bugs.

The bar must name its methods, numbers and tools; be strictest on the data path; be enforced in CI and in the
agent workflow, not only written down; stay fast enough for a solo project; and work within Tauri's limits
(WebDriver does not run against macOS webviews).

## Options

### A. Pragmatic baseline

Unit and integration tests, a coverage floor from `cargo-llvm-cov` and Vitest, a regression test for every bug
fix. Cheap and fast. Coverage only shows that code ran, not that a test would notice it breaking, so a high
number can hide tests that assert nothing.

### B. Risk-tiered

The same floor everywhere, plus stricter methods where a bug costs most: mutation testing (the tool changes
the code at random and a good test then fails), property-based tests (random inputs against stated rules) and
golden fixtures on the data path. Lighter for the UI and none for tools. It costs longer CI runs and some
upkeep, and it needs a rule for mutants that cannot be caught.

### C. Aviation-style

Requirements traceability, MC/DC coverage and independent verification. Far heavier than a household app
needs.

## Decision

We chose **Option B**, as advised.

| Tier | Parts | Required |
|---|---|---|
| 1 | `trove-core`, `trove-store`, migrations, operation log | line coverage ≥ 90%; mutation score ≥ 85%; `proptest` properties for every validation and ordering rule and for replay; a golden fixture per `op_version` |
| 2 | Shell commands | line coverage ≥ 80%; every command tested through mock IPC, including its error codes |
| 3 | UI | Vitest and Testing Library for components and screens; an accessibility check; a browser smoke test against a stand-in for the Rust side; `fast-check` for pure functions |
| 4 | Tools, docs | none |

- **Measurement.** `cargo-llvm-cov` for coverage (Rust branch coverage needs nightly, so line and region
  coverage are used), `cargo-mutants` for mutation, Vitest coverage for the UI. CI runs mutation on the changed
  lines of a pull request and on everything weekly.
- **Ratchet.** Thresholds sit in CI and can be raised but never lowered. A mutant that cannot be caught gets a
  skip marker with a written reason.
- **Bugs.** A fix starts with a failing test that reproduces it.
- **Property tests** are bounded in cases, and the inputs they find are committed so they stay reproducible.
- **Human review.** Every pull request body has a "Try to break this" list: the fragile spots, the test for
  each, and what is not covered. The user replies with cases, which become tests.
- **Reference notes.** `docs/tests/` holds how each tier is tested, how to run the measurements and the fragile
  places with the test that guards each. They are reference notes, not decisions or plans, so they do not
  conflict with [1](0001-record-decisions-as-adr-files.md). `ARCHITECTURE.md` only links to them.
- **Enforcement.** CI holds the numbers, since it is the only layer that cannot be skipped. `CLAUDE.md` carries
  the short rule, the `/add-tests <issue> "<what to test>"` skill carries the procedure (audit what exists,
  report each case as covered, partly covered, not covered or no longer relevant, discuss, then write), and
  `/start-issue` and `/open-pr` ask for the tests and the "Try to break this" list.

This rules out aviation-style verification and a single flat coverage number.

## Consequences

- The data path gets evidence that its tests catch bugs, not only that they run, and every change shows its
  fragile spots to a human.
- CI gets longer and needs `cargo-llvm-cov` and `cargo-mutants`. Rust gets no branch coverage.
- Some mutants cannot be caught, which is why the score is 85%, not 100%. The number can be raised later.
- `docs/tests/` must stay current; `/add-tests` updates it.
- Follow-ups under #74: CI measurement and ratchet; property and mutation tests for the data path; the
  frontend runner, component tests, accessibility check and browser smoke test; `docs/tests/` and the theme
  guards. #89 builds the `/add-tests` skill.
