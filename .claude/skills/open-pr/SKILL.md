---
name: open-pr
description: Run the CI checks locally, push the current branch and open a pull request with a Conventional Commit title. A sub-issue's pull request targets its parent's integration branch; an integration branch gets the final pull request into main.
disable-model-invocation: true
---

# Open a pull request for the current branch

1. **Branch and base.** `git branch --show-current` must match `<type>/<issue>-<short-name>`. On `main`,
   stop. Take `<issue>` from the name. The base is the integration branch of the issue's parent
   (`gh api repos/{owner}/{repo}/issues/<issue>/parent -q .number`, an error means none), found with
   `git fetch origin` and `git branch -r --list "origin/*/<parent>-*"`; with no parent it is `main`. If the
   parent has no integration branch yet, create it as `/start-issue` does. Then decide the kind
   (CONTRIBUTING.md, "Parent issues"):
   - **Final**: the issue has sub-issues (`gh api repos/{owner}/{repo}/issues/<issue>/sub_issues -q
     '.[].number'` is not empty), so this is its integration branch.
     - Every sub-issue that is still open needs a merged pull request into this branch
       (`gh pr list --state merged --base <branch> --json number,headRefName`, a head branch starting
       `<type>/<sub-issue>-`). If one has none, stop and list them. If one has a merged pull request
       but is still open, close it as in step 8. Closed sub-issues, such as ADRs that went to `main`,
       are skipped.
     - If `git rev-list --count HEAD..origin/<base>` is not 0, ask, then merge `origin/<base>` into the
       branch (a merge, not a rebase: others have branches cut from it) and push before going on.
   - **Sub-issue**: the issue has no sub-issues but has a parent. Its pull request goes to that parent's
     integration branch.
   - **Plain**: neither. The base is `main`.
2. **Checks.** Run what CI runs. If anything fails, stop, show the failure and offer to fix it.
   ```sh
   (cd apps/desktop && npm run check && npm run build)
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check      # if cargo-deny is installed; CI runs it either way
   ```
   If the diff touches Rust, also run the measurements of ADR 17 and show the numbers. Tier 1 code
   (core, store, migrations, operation log) is also mutation-tested. If a number is below its
   threshold, stop, as for a failing check.
   ```sh
   cargo llvm-cov --workspace --json --output-path /tmp/cov.json
   node tools/quality/check.mjs coverage /tmp/cov.json
   git diff <base>...HEAD > /tmp/pr.diff && cargo mutants -p <crate> --in-diff /tmp/pr.diff   # Tier 1 crates only
   node tools/quality/check.mjs mutation <crate> mutants.out/outcomes.json
   ```
3. **Scope.** `git diff <base>...HEAD --stat`. Flag files that don't belong to the issue (for a final
   pull request: to the parent and its sub-issues), and new literal colours, fonts, radii or durations in
   `apps/desktop/src` (see docs/themes/README.md). Also flag strings that bypass `t()` (see
   docs/languages.md) and structural changes that have no ADR. Ask before going on if any are found.
4. **Commit.** If there are uncommitted changes, ask whether to commit them.
5. **Title.** `<type>(<scope>): <summary>` per [Conventional Commits](https://www.conventionalcommits.org/).
   The type comes from the branch. The scope is the area touched, optional. The summary is imperative,
   lower case and has no full stop. Get it from `gh issue view <issue> --json title`. This title becomes the squash commit on the base.
6. **Open.**
   ```sh
   git push -u origin HEAD
   gh pr create --base <base> --title "<title>" --body-file <filled copy of .github/pull_request_template.md>
   ```
   The first line of the body depends on the kind. GitHub closes issues only when a pull request
   reaches `main`, and needs one keyword per issue.
   - **Plain**: `Closes #<issue>`.
   - **Sub-issue**, or a **final** pull request whose base is not `main`: `Part of #<parent>. Refs
     #<issue>`. The issue is closed by hand after the merge (step 8).
   - **Final** into `main`: `Closes #<issue>`, then one `Closes #<n>` line for each descendant that
     is still open (normally none: they were closed when their pull requests merged).

   Then one or two sentences of what and why, a **Try to break this** section and the checklist items
   that hold ticked. The section lists the fragile spots of the change, the test that guards each and
   what is not covered (take them from `/add-tests` if it was used, and from the diff if not). It
   ends by asking the reviewer for cases to add. If the diff has no code to test, write "Nothing to
   break: docs and tooling only."
7. Give the user the PR URL. Do not merge unless the user allows it. A final pull request is merged
   with its branch deleted.
8. **Close the issue** once a pull request whose base is not `main` is merged, because GitHub will not:
   `gh issue close <issue> --reason completed --comment "Merged into <base> in #<pr>."`. A pull request
   into `main` closes its issue through `Closes`.
