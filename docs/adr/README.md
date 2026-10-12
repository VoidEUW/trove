# Architecture decision records

ADRs are the only design documents written in this repository. Plans, features and bugs live in
GitHub issues. An ADR records one structural decision: the question, the options, what was chosen
and what follows from it.

## How an ADR comes to be

1. Someone opens an issue with the **Architecture decision** template. It gets the `adr` label and
   states the question, not the answer.
2. Whoever takes it runs `/solve-adr <issue>` in Claude Code, or follows the same steps by hand:
   branch `docs/<issue>-adr-<short-name>`, work through context, options and consequences, and copy
   [template.md](template.md) to the next free number.
3. The pull request adds the ADR, adds a row to the index below, and updates the matching section
   of [ARCHITECTURE.md](../../ARCHITECTURE.md). Its body says `Closes #<issue>`, and it is
   squash-merged like every other pull request.

## Rules

- Files are named `NNNN-kebab-case-title.md`, numbered in order, never renumbered.
- Status is `Proposed`, `Accepted`, or `Superseded by NNNN`. It is `Accepted` once the PR is merged.
- An accepted ADR is not rewritten. To change a decision, write a new ADR that supersedes it, and
  set the old one's status to `Superseded by NNNN`.
- Keep it short. If an ADR needs more than a page, it is probably more than one decision.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-record-decisions-as-adr-files.md) | Decisions are ADR files; plans live only in issues | Accepted |
| [0002](0002-desktop-app-with-tauri-and-svelte.md) | Trove is a local-first Tauri 2 desktop app with a Svelte 5 UI | Accepted |
| [0003](0003-crate-map.md) | The Rust side is three crates whose dependencies point one way | Accepted |
| [0004](0004-typed-command-boundary.md) | The UI calls Rust only through typed Tauri commands and events | Accepted |
| [0005](0005-domain-model.md) | The domain is a normalised relational model of spaces, places and items | Accepted |
| [0006](0006-item-kinds-as-data.md) | Item kinds are user-editable data with typed field definitions | Accepted |
| [0007](0007-room-layout-solver.md) | Rooms attach to each other's walls, and one solver in the core places them | Accepted |
| [0008](0008-sqlite-storage.md) | Data lives in one SQLite database in a local data folder | Accepted |
| [0009](0009-operation-log-and-sync.md) | Every write records a field-level operation; devices sync by exchanging operation files | Accepted |
| [0010](0010-operations-as-source-of-truth.md) | The operation log is the source of truth; SQLite is a rebuildable projection | Accepted |
| [0011](0011-search-with-fts5.md) | Search uses SQLite FTS5 and returns where each hit is | Proposed |
| [0012](0012-attachments-and-secrets.md) | Attachments are stored by content hash; secrets are encrypted with a key in the OS keychain | Proposed |
| [0013](0013-threlte-floor-plan-scene.md) | One Threlte scene draws the floor plan in both 2D and 3D | Accepted |
| [0014](0014-ui-architecture-and-tokens.md) | The UI is a SvelteKit SPA whose look comes only from the Heaven tokens | Accepted |
| [0015](0015-ui-languages.md) | The interface is in English and German from the first component | Accepted |
| [0016](0016-operation-encoding-and-clock-format.md) | Operations are JSON patches ordered by a packed 64-bit hybrid logical clock | Accepted |
