# 8. Data lives in one SQLite database in a local data folder

- Status: Accepted
- Date: 2026-10-11
- Origin: spec "Storage" in the first repository
- Issue: #1

## Context

Trove works offline ([2](0002-desktop-app-with-tauri-and-svelte.md)), needs relational queries
([5](0005-domain-model.md)) and full-text search, and must survive crashes. A synced folder such as
iCloud Drive corrupts a live SQLite file when it syncs the database and its journal separately.

## Decision

- **Folder**: `~/.trove` on macOS and Linux, `%APPDATA%\trove` on Windows; `TROVE_HOME` overrides it.
  `trove-desktop` resolves the path and hands it to `trove-store`, which never guesses it.

  | Entry | Contents |
  |---|---|
  | `trove.db` | SQLite in WAL mode |
  | `files/` | Attachments by hash ([12](0012-attachments-and-secrets.md)) |
  | `backups/` | Snapshots taken with the SQLite backup API |
  | `config.toml` | Settings that are not secrets |
  | `logs/` | Rotating log files |

- **Engine**: `rusqlite` with bundled SQLite (includes FTS5). Synchronous; no async runtime.
  One writer connection behind a mutex, read connections as needed, foreign keys on.
- **Migrations**: `rusqlite_migration`, SQL files embedded in `trove-store`, forward-only, versioned
  through `user_version`, run on open before any other access. A database newer than the app is
  refused, not opened.
- The live database never lives in a synced folder.

This rules out an ORM, `sqlx`, an async runtime in the store, and another database engine.

## Consequences

- Zero setup, one file to back up, fast local queries.
- Tests set `TROVE_HOME` to a temporary directory and never touch real data.
- Downgrading the app after a migration is impossible; backups before migrating are mandatory.
- With [10](0010-operations-as-source-of-truth.md), `trove.db` is a rebuildable projection of the
  operation log rather than the only live state.
- Follow-ups: data-folder resolution, connection setup, first migration, automatic backups.
