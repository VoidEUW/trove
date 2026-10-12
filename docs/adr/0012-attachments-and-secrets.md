# 12. Attachments are stored by content hash; secrets are encrypted with a key in the OS keychain

- Status: Proposed
- Date: 2026-10-11
- Origin: spec "Attachments and secrets" in the first repository
- Issue: #1

## Context

Items carry photos, manuals, receipts and scans. Licence keys are part of the first version
(decided 2026-10-12) and are shown masked and revealed on demand. Both must be movable by sync
later ([9](0009-operation-log-and-sync.md)).

## Decision

- **Files**: stored under `files/ab/cdef…` by SHA-256; table
  `attachments(item, hash, filename, mime, size)`. Identical files are stored once. Thumbnails are
  generated on import. Files sync separately by hash.
- **Secrets**: one data key per installation, kept in the OS keychain through the `keyring` crate
  (macOS Keychain, Windows Credential Manager). Each secret value is encrypted with
  XChaCha20-Poly1305 and stored in the database. Secrets are never indexed or logged.

This rules out storing each secret in the keychain directly (cannot sync, size limits) and an app
password at every start.

## Consequences

- Deduplication for free; a hash is a stable sync key.
- **Open point**: a second device needs the same data key, through iCloud Keychain or a one-time
  export. Without it, synced secrets cannot be read there. Decide before sync.
- Losing the keychain entry loses the secrets; key export belongs in backup.
- Follow-ups: attachment import and thumbnailing, keyring wrapper, secret field UI.
