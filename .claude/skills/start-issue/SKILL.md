---
name: start-issue
description: Claim a GitHub issue and create its branch, cut from the parent's integration branch when the issue is a sub-issue. Use when the user wants to start working on an issue by number.
argument-hint: <issue-number>
disable-model-invocation: true
---

# Start issue #$ARGUMENTS

Follow CONTRIBUTING.md. Stop and report to the user at any failed check; do not work around it.

1. **Read it.** `gh issue view $ARGUMENTS --json number,title,body,labels,assignees,state`.
   - Closed → stop.
   - Assigned to someone other than `gh api user -q .login` → stop and say who has it.
   - No acceptance criteria in the body → stop. Suggest what they could be and offer to comment them on the issue.
   - Label `adr` → stop and tell the user to run `/solve-adr $ARGUMENTS` instead.
   - It has sub-issues (`gh api repos/{owner}/{repo}/issues/$ARGUMENTS/sub_issues -q '.[].number'` is not
     empty) → stop. A parent issue has no work of its own; list its open sub-issues in order and
     suggest starting the first one.
   - Find its parent: `gh api repos/{owner}/{repo}/issues/$ARGUMENTS/parent -q .number` (an error means
     none). Call it `<parent>`.
2. **Claim it.** `gh issue edit $ARGUMENTS --add-assignee @me`.
3. **Branch.** The type comes from the labels: `bug` → `fix`, `docs` → `docs`, `chore` → `chore`,
   `refactor` → `refactor`, otherwise `feat`. The short name is 2–4 kebab-case words from the title.
   If the working tree is dirty, ask before switching.

   *No parent:*
   ```sh
   git switch main && git pull --ff-only
   git switch -c <type>/$ARGUMENTS-<short-name>
   ```

   *With a parent*, the work goes through the parent's integration branch `<type>/<parent>-<short-name>`
   (CONTRIBUTING.md, "Parent issues"). Find it with `git fetch origin` and
   `git branch -r --list "origin/*/<parent>-*"`.
   - If it exists, use it as `<base>`.
   - If it does not, create it. Its type and short name come from the parent issue's labels and title,
     by the same rules as above. It is cut from the parent's own integration branch if the parent has a
     parent, from `main` otherwise (find or create that one the same way). Tell the user you are
     creating it.
     ```sh
     git push origin origin/<its-base>:refs/heads/<type>/<parent>-<short-name>
     ```
   Then cut the sub-issue's branch from it:
   ```sh
   git fetch origin
   git switch -c <type>/$ARGUMENTS-<short-name> --no-track origin/<base>
   ```
4. **Orient.** Read the parts of ARCHITECTURE.md and docs/adr/ the issue touches, and the parent issue
   if there is one. For UI work, read docs/themes/system.md and the active theme
   (docs/themes/README.md). For new strings, read docs/languages.md.
5. **Report.** Give the branch name and the branch its pull request will target (`main`, or the
   integration branch), the acceptance criteria as a checklist, and a plan of at most 5 steps. The
   plan lists the tests the work needs: the tier of each piece of code and the methods ADR 17 requires
   for it (`/add-tests` writes them).
   If the issue needs a structural decision that no ADR covers, say so and suggest opening an `adr` issue
   rather than deciding it in code. Wait for the go-ahead before writing code.

When the work is done, `/open-pr` opens the pull request.
