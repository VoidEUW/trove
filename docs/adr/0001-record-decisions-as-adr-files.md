# 1. Decisions are ADR files; plans live only in issues

- Status: Accepted
- Date: 2026-10-12
- Origin: replaces the no-docs rule of the first repository's `CLAUDE.md`
- Issue: #1

## Context

The first repository kept every decision in closed spec issues and banned other docs. When the
repository was deleted, the issues and their decision comments went with it; only the local copy of
`ARCHITECTURE.md` survived. Decisions change rarely and agents need them on disk. Plans, bugs and
features change all the time, and copies of them in files drift from the issues.

## Decision

- Every architectural decision is a file `docs/adr/NNNN-kebab-title.md` written from
  `docs/adr/template.md`. Numbers are sequential and never reused.
- A spec issue holds the discussion. The ADR is merged in the pull request that closes it. If the
  two disagree, the ADR wins.
- An accepted ADR is not rewritten. Only its status line changes; a changed decision is a new ADR
  that supersedes the old one.
- `ARCHITECTURE.md` describes how the parts fit *now* and cites ADRs for why.
- The visual system lives in `docs/themes/<name>/` (`theme.md`, `theme.png`, `icons.svg`).
- Plans, tasks, bugs, features, progress and implementation steps live only in issues.

This rules out plan documents, TODO files, roadmaps, hand-written changelogs and design notes in the
repository.

## Consequences

- The reasons behind the code survive a lost repository or tracker.
- An agent reads ADRs before code; `/start-issue` must load the ADRs an issue cites.
- Issues are still not in git: losing the repository loses the plan. A periodic export of issues
  (`gh issue list --json …`) is worth a chore issue.
- Follow-ups: move the theme files into `docs/themes/<name>/` and adapt the theme tooling to it.
