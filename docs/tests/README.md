# Tests

Reference notes for the quality bar of [ADR 17](../adr/0017-risk-tiered-quality-bar.md). They describe
how things are tested and measured today; they are not decisions or plans (those are ADRs and issues).
Update them when you add or remove a fragile place: `/add-tests` does this as its last step.

| Tier | Code | Note |
|---|---|---|
| 1 | `trove-core`, `trove-store`, migrations, operation log | [data-path.md](data-path.md) |
| 2 | Shell commands in `apps/desktop/src-tauri` | [commands.md](commands.md) |
| 3 | UI in `apps/desktop/src` | [ui.md](ui.md) |
| 4 | Tools and docs | No tests required |

The places where a change is most likely to break something, and the test that guards each, are in
[fragile-places.md](fragile-places.md).

## Running everything

From the repo root unless noted.

```sh
cargo test --workspace                                    # Rust unit, integration and property tests
(cd apps/desktop && npm test)                             # Vitest: API layer, i18n, components, screens, axe
(cd apps/desktop && npm run test:e2e)                     # Playwright smoke tests (first: npx playwright install chromium)
node tools/theme-check/check.mjs                          # tokens.css matches the theme; no literal values in src
```

## Measuring

The numbers live in [`tools/quality/thresholds.json`](../../tools/quality/thresholds.json). They can be
raised but never lowered: the `ratchet` job in CI compares the file with the branch a pull request
targets.

```sh
cargo llvm-cov --workspace --json --output-path /tmp/cov.json
node tools/quality/check.mjs coverage /tmp/cov.json       # line coverage per tier

cargo mutants -p trove-core                               # or -p trove-store; add --file <path> --re '<Type>::<fn>' to narrow
node tools/quality/check.mjs mutation trove-core mutants.out/outcomes.json
```

| Measure | Where | Minimum |
|---|---|---|
| Line coverage, `core` | `crates/trove-core/src/` | 90% |
| Line coverage, `store` | `crates/trove-store/src/` | 90% |
| Line coverage, `commands` | `commands.rs`, `error.rs`, `events.rs`, `items.rs` in the shell | 80% |
| Mutation score, `trove-core` and `trove-store` | `cargo mutants` | 85% (timeouts count as caught) |

- Rust branch coverage needs nightly, so line coverage is used. It counts the inline `#[cfg(test)]`
  code, which makes the numbers slightly high.
- CI runs coverage in the `rust` job. Mutation runs on the changed lines of a pull request (job
  `mutation`, report only) and on everything every Monday (`.github/workflows/mutation.yml`).
- A mutant that no test can catch is skipped in
  [`.cargo/mutants.toml`](../../.cargo/mutants.toml) with a comment giving the reason. Skip it only
  when the change makes no observable difference; otherwise add a test.
- The `mutation`, `e2e` and `theme` jobs are not required status checks yet; the `protect-main`
  ruleset lists the required ones.
