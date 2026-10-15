Closes #

## What and why

<!-- One or two sentences. The title is the squash commit: `type(scope): summary`. -->

## Try to break this

<!-- The fragile spots of this change, the test that guards each, and what is not covered. Reviewers: reply with cases to add. Docs and tooling only: "Nothing to break". -->

## Checklist

- [ ] CI is green (`npm run check`, `cargo fmt`, `cargo clippy`, `cargo test`, `cargo deny check`)
- [ ] Structural change? There is an accepted ADR for it
- [ ] New UI uses components from `docs/themes/system.md` and only theme tokens
- [ ] New strings go through `t()` and exist in `en.json` (`docs/languages.md`)
