# CLAUDE.md

Trove catalogues a household's belongings and shows where they are on a 2D and 3D floor plan. It is
a local-first Tauri 2 desktop app: a Svelte 5 frontend in `apps/desktop`, the Tauri shell crate in
`apps/desktop/src-tauri` and logic crates in `crates/`. No server, no account, no network.

Before any work, read the issue, `ARCHITECTURE.md` and the ADRs the touched sections cite. Read
`CONTRIBUTING.md` before git work.

## Commands

```sh
cd apps/desktop && npm ci           # once
npm run tauri dev                   # run the app (from apps/desktop)
npm run check && npm run build      # frontend type-check and build (from apps/desktop; build is needed before cargo clippy)
cargo fmt --all                     # from the repo root
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check                    # advisories, licenses, sources
node tools/theme-preview/render.mjs <theme>   # re-render a theme's .png after editing its .md
```

## Rules

- **Layering.** `trove-core` holds domain types, validation, the layout solver and ports (traits),
  and does no IO. `trove-store` holds SQLite, migrations, the operation log, files and search, and
  implements the ports. Neither depends on Tauri. The shell only translates between the UI and the
  crates. The UI does no I/O: components call wrappers in `apps/desktop/src/lib/api/` and never
  `invoke` directly.
- **Data.** Every mutation goes through the store's service layer and writes an operation in the
  same transaction. The operation log is the source of truth and the tables are a projection; never
  fix data with direct SQL. Migrations are forward-only. A changed operation type needs an
  `op_version` upcaster and golden fixtures. Ids are UUIDv7 generated in core; lengths are integer
  millimetres. Room positions come only from the layout solver.
- **Boundary.** Types that cross to the UI derive `ts_rs::TS`; regenerate the bindings and never
  write those TypeScript types by hand. Crates return their own `thiserror` types; only the shell
  maps them to `AppError { code, message }`.
- **UI.** Build components from the catalogue in `docs/themes/system.md`, in
  `apps/desktop/src/lib/components/`, named after their id. The active theme
  (`docs/themes/README.md`) sets each component's look. Never write a literal colour, font, radius
  or duration in `apps/desktop/src`. Use the tokens in `src/lib/theme/tokens.css`, which is
  generated, so don't edit it. Use `/apply-theme` for UI work.
- **Strings.** Every user-visible string is a key in `apps/desktop/src/lib/i18n/en.json`, translated
  in `de.json`, and used with `t()`. Follow the key rules in `docs/languages.md`. Text a user types
  is stored as typed and never translated.
- **Secrets.** Never logged, never indexed, revealed only on demand.
- **Decisions.** Do not make a structural decision in code. If no ADR covers it, stop and suggest an
  `adr` issue. ADRs are the only design docs; don't create other planning, status or TODO files.
- **Issues.** Issues are the plan and guide the work step by step. An issue whose work has several
  distinct parts, or needs more than a few prompts, is split into sub-issues; an issue made of small
  steps that each fit in one prompt is not.
- **Git.** One issue, one branch `<type>/<issue>-<short-name>`. Conventional Commits. PRs are
  squash-merged, and the PR title is the commit. Never push to `main`. Never merge a PR yourself.
  A parent issue has an integration branch `<type>/<parent>-<short-name>`: its sub-issues are cut
  from it and their PRs target it (`Part of #<parent>`, `Refs #<sub-issue>`). Close a sub-issue by
  hand when its PR is merged; one final PR into `main` closes the top parent. An `adr` sub-issue
  goes straight to `main`. See `CONTRIBUTING.md`.
- **Rust.** Edition 2024. No `unsafe`. Clippy warnings fail CI. No `unwrap` outside tests; `expect`
  only at startup in the shell.
- **Tests.** Core is pure and covered by unit tests; the solver also by property tests. Store tests
  use a temporary `TROVE_HOME` and never touch real data.

## Skills

`/start-issue <n>`, `/solve-adr <n>`, `/open-pr`, `/apply-theme [name]`, `/add-language <code>`,
`/import-idea <file>`.
They live in `.claude/skills/`.
