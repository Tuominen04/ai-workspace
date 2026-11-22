# Repository settings

The GitHub settings that protect this repository once it is public. Files in the repository
(issue forms, `CODEOWNERS`, Dependabot, the CodeQL, Audit and Scorecard workflows) do part of the
work; the rest is set by the maintainer in the repository's **Settings**, in the order below.
Menu names follow GitHub's current layout and may move.

## Before making it public

- [ ] The history has no secrets: checked on 2026-11-22 for private keys and common token
  formats (GitHub, OpenAI, AWS, Slack, Google) across every commit; none found.
- [ ] Author email: every commit shows its author's address. Optional, for future commits:
  **Settings (your account) → Emails → Keep my email address private** and **Block command line
  pushes that expose my email**, then `git config --global user.email
  <id>+<user>@users.noreply.github.com` with the address that page shows.
- [ ] Delete merged branches: **Code → Branches → Stale/All**, or `git push origin --delete
  <branch>` for each.

## General

- [ ] **Features**: Issues on; **Discussions** on (then in Discussions keep the categories
  **Q&A** and **Ideas**: the issue forms link to them); Wiki off; Projects off unless used.
- [ ] **Pull requests**: allow merge commits (as so far), **Automatically delete head
  branches** on, **Always suggest updating pull request branches** on.
- [ ] **Releases**: **Enable release immutability** on: a published release's files and tag
  cannot be changed afterwards. The release workflow also attests each package's build
  provenance (see "Checking where it was built" in `docs/release.md`).

## Rules → Rulesets

- [ ] **New branch ruleset** `main`, enforcement **Active**, target **Include default branch**.
  - **Bypass list**: Repository admin, **Always** (you: push to `main` and merge your own pull
    requests).
  - **Restrict deletions** and **Block force pushes**.
  - **Require a pull request before merging**: 1 approval, **Require review from Code Owners**,
    **Dismiss stale approvals**, **Require conversation resolution**. Only you are a code owner,
    so nobody else's pull request can be merged without your review; your own you merge with
    the bypass.
  - **Require status checks to pass**, **Require branches to be up to date**: `check`,
    `desktop-shell`, `desktop-interface`, `desktop-e2e` (CI), `rust`, `npm` (Audit); after the
    repository is public and CodeQL has run once, also `analyze (rust)` and
    `analyze (javascript-typescript)`.
- [ ] **New tag ruleset** `release tags`, **Active**, target pattern `v*`.
  - **Bypass list**: Repository admin, **Always**.
  - **Restrict creations**, **Restrict updates**, **Restrict deletions**. A `v*` tag starts the
    release workflow, so only you can create one.

## Actions → General

- [ ] **Fork pull request workflows from outside collaborators**: **Require approval for all
  external contributors**: their CI runs only after you look at the change.
- [ ] **Workflow permissions**: **Read repository contents and packages permissions**; leave
  **Allow GitHub Actions to create and approve pull requests** off. The release workflow asks
  for `contents: write` itself.

## Advanced Security (Code security)

- [ ] **Private vulnerability reporting** on (`SECURITY.md` points to it).
- [ ] **Dependency graph** on; **Dependabot alerts** on (malware alerts for npm packages come
  with them where GitHub offers them); **Dependabot security updates** on. Version updates are
  configured in `.github/dependabot.yml`.
- [ ] **Secret scanning** on, with **Push protection** on.
- [ ] **Code scanning**: leave **Default setup** off; `.github/workflows/codeql.yml` is the
  advanced setup and runs once the repository is public.

## Collaborators

- [ ] No collaborators: only the owner can change settings, rulesets and secrets.

## Make it public

- [ ] **General → Danger Zone → Change visibility → Public.**
- [ ] Afterwards: the CodeQL and Scorecard workflows start on the next push to `main` (or run
  them from **Actions**); check **Security → Code scanning** and add the two CodeQL checks to the
  `main` ruleset.
