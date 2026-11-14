# Tasks: Release Packaging

**Input**: Design documents from `/specs/007-release-packaging/`

**Prerequisites**: [plan.md](./plan.md), [spec.md](./spec.md), [research.md](./research.md), [data-model.md](./data-model.md), [contracts/](./contracts/), [quickstart.md](./quickstart.md)

**Tests**: Included for the rename. Per `CONTRIBUTING.md`'s Testing Guidelines, a behavior change needs a happy path and a critical failure path — covered by `assert_cmd` integration tests in `tests/app_version_tests.rs`. The package and the workflow cannot be tested by `cargo test`; they are verified by the workflow's own smoke test and by the `quickstart.md` scenarios, which appear below as explicit validation tasks.

**Organization**: Tasks are grouped by user story (P1/P2/P3 from spec.md). The rename and the license are prerequisites of every story and sit in Setup and Foundational.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: Which user story the task belongs to (US1, US2, US3)
- **(maintainer)**: Needs `sudo`, a tag push, or a publish — done by or with the maintainer's explicit go-ahead

## Path Conventions

Single Rust CLI project: `src/`, `tests/` at the repository root (flat module layout, no new modules). One new workflow under `.github/workflows/`.

---

## Phase 1: Setup

**Purpose**: Version and license, which every later task builds on.

- [ ] T001 Set `version = "0.6.0"` in `Cargo.toml` and run `cargo build` so `Cargo.lock` records the same version (research.md Decision 5)
- [ ] T002 [P] Create `LICENSE-MIT` and `LICENSE-APACHE` at the repository root by copying the texts from `/home/iamtuominen/git/learning-corner/LICENSE-MIT` and `/home/iamtuominen/git/learning-corner/LICENSE-APACHE`; keep the MIT copyright line as `Copyright (c) 2026 Arttu Tuominen` (research.md Decision 7)
- [ ] T003 Add `license = "MIT OR Apache-2.0"` to the `[package]` table in `Cargo.toml` (depends on T001)

**Checkpoint**: `cargo build` succeeds; `cargo metadata --no-deps --format-version 1` reports version `0.6.0` and the license.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Rename the command from `ai` to `local-ai` everywhere it names itself, per [contracts/cli-naming.md](./contracts/cli-naming.md). The package (US1) and the workflow's smoke test (US2) both depend on the new name.

**⚠️ CRITICAL**: No user story task can begin until this phase is complete.

### Tests (write first; they must fail before T008)

- [ ] T004 In `tests/app_version_tests.rs`, change `Command::cargo_bin("ai")` to `Command::cargo_bin("local-ai")`, make the existing version test expect exactly `local-ai <CARGO_PKG_VERSION>`, and add a test `help_names_the_command_local_ai` asserting that `--help` exits `0`, contains the lines `  local-ai init`, `  local-ai search --query TEXT [--project PROJECT] [--limit N]` and `  local-ai --help`, and contains no line that starts with two spaces followed by `ai ` (the old usage form)
- [ ] T005 [P] In `tests/search_tests.rs`, change all four `Command::cargo_bin("ai")` calls to `Command::cargo_bin("local-ai")` and update any assertion that quotes `ai permission view` to `local-ai permission view`

### Implementation

- [ ] T006 In `Cargo.toml`, change the `[[bin]]` target's `name` from `"ai"` to `"local-ai"` (path stays `src/main.rs`; the `[package] name` stays `ai-workspace`)
- [ ] T007 In `src/lib.rs`, add `pub const COMMAND_NAME: &str = "local-ai";` with a doc comment stating it is the single source for the command's name in every user-facing message (research.md Decision 4)
- [ ] T008 In `src/app.rs`, build every string that names the command from `crate::COMMAND_NAME`: all `usage: ai ...` error messages, every line of `print_help` (including `Show the local-ai binary version`), the unknown-command hint pointing at `--help`, and the `print_version` line (`"{COMMAND_NAME} {version}"`). Where one usage string is repeated several times, define it once per subcommand (a small helper returning the `String`) instead of repeating the `format!` (depends on T007)
- [ ] T009 [P] In `src/error.rs`, build the `usage: ai search --query TEXT [--project PROJECT] [--limit N]` message from `crate::COMMAND_NAME` (depends on T007)
- [ ] T010 [P] In `src/session.rs`, build from `crate::COMMAND_NAME` the six labels passed to `enforce_validation_gate` (`search`, `project create`, `project show`, `permission grant`, `permission revoke`, `permission view`) and the `Create a project directory there and rerun ai.` hint (depends on T007)
- [ ] T011 [P] In `src/ui.rs`, build from `crate::COMMAND_NAME` the validation-gate lines (`Gated commands blocked: ...`, ``Run `ai validate` ...`` in both places) and the search no-results hint (``check `ai permission view` ...``); these are string changes only — no logic or filesystem access is added (depends on T007)
- [ ] T012 [P] In `src/workspace.rs`, build from `crate::COMMAND_NAME` the four `suggested_fix` strings that name `ai init` or `ai validate` (depends on T007)
- [ ] T013 Run `cargo test`; then run `grep -nE '"[^"]*\bai (init|status|validate|config|project|permission|search|--)' src/*.rs` and `grep -n 'rerun ai\|`ai ' src/*.rs` and confirm both print nothing; fix any literal that remains (depends on T004–T012)

**Checkpoint**: `cargo run -- --version` prints `local-ai 0.6.0`; `target/debug/local-ai` exists and a clean build produces no `target/debug/ai`; all tests pass. This is `quickstart.md` Scenario 1.

---

## Phase 3: User Story 1 - Install a released version on Fedora (Priority: P1) 🎯 MVP

**Goal**: An `.rpm` that installs `local-ai` on Fedora with `dnf`, upgrades and removes cleanly, declares its license, and never touches the workspace folder.

**Independent Test**: Build the package locally, inspect it, install it with `dnf`, run `local-ai --version` in a new terminal, remove it, and confirm the workspace folder is unchanged — `quickstart.md` Scenarios 3 and 4.

### Implementation for User Story 1

- [ ] T014 [US1] Add a `[package.metadata.generate-rpm]` table to `Cargo.toml` per [contracts/rpm-package.md](./contracts/rpm-package.md): `name = "local-ai"`; built-in dependency detection (`auto-req = "builtin"`); and exactly three assets — `target/release/local-ai` → `/usr/bin/local-ai` mode `755`, `LICENSE-MIT` → `/usr/share/licenses/local-ai/LICENSE-MIT` mode `644`, `LICENSE-APACHE` → `/usr/share/licenses/local-ai/LICENSE-APACHE` mode `644`. Check the `cargo-generate-rpm` 0.21.1 README for the exact key names and for a way to flag the two texts as license files, and use it if one exists (depends on T002, T003, T006)
- [ ] T015 [US1] Install the tool with `cargo install --locked cargo-generate-rpm@0.21.1`, then run `quickstart.md` Scenario 3 (`cargo build --release --locked`, `cargo generate-rpm`, `rpm -qip`, `rpm -qlp`, `rpm -qp --scripts`, `rpm -qLp`) and adjust `Cargo.toml` until name, version, license, summary and the three-file list match the contract and there are no install scripts. If the tool cannot mark license files, remove the `rpm -qL` claim from `contracts/rpm-package.md` and `quickstart.md` rather than working around it (depends on T014)
- [ ] T016 [US1] (maintainer) Run `quickstart.md` Scenario 4 on the Fedora machine: checksum the workspace folder, `sudo dnf install` the package, confirm `local-ai --version` in a new terminal, perform the upgrade check with a temporarily higher version, `sudo dnf remove local-ai`, and confirm the workspace checksum is unchanged. Requires `sudo`; ask the maintainer to run it and record the result (depends on T015)
- [ ] T017 [P] [US1] Update `README.md`: replace every `ai ...` command in the "Workspace Manager Commands" and "Permission Commands" sections with `local-ai ...`; add an **Install** section (download `local-ai-<version>.x86_64.rpm` from the release, `sudo dnf install ./local-ai-<version>.x86_64.rpm`, set `AI_WORKSPACE_HOME` if the workspace is not at `~/AI`, delete an old hand-copied `~/.local/bin/ai`); add a **License** section ("Dual licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.")
- [ ] T018 [P] [US1] Replace the command name `ai` with `local-ai` in every command example and inline reference in `docs/workspace.md` and `docs/permissions.md` (leave `AI_WORKSPACE_HOME` and `~/AI` unchanged)

**Checkpoint**: A locally built `local-ai-0.6.0-1.x86_64.rpm` installs, upgrades and removes cleanly on Fedora, and the docs tell a user how to install it.

---

## Phase 4: User Story 2 - Cut a release by pushing a version tag (Priority: P2)

**Goal**: Pushing `v<version>` builds, tests, packages, smoke-tests and attaches the package and checksum file to a draft release, with no other manual step; wrong tags and failing tests produce nothing.

**Independent Test**: Push a correct tag and see a draft with two assets; push a mismatched tag and a tag off `main` and see no release — `quickstart.md` Scenarios 5 and 6 (run after merge, T028–T029).

### Implementation for User Story 2

- [ ] T019 [US2] Run `quickstart.md` Scenario 2 (`PATH=/usr/bin:/bin cargo test --locked`, which hides `/usr/local/bin/ollama`) and confirm every test passes without a local AI runtime; if a test needs the runtime, fix the test, not the workflow (research.md Decision 8)
- [ ] T020 [US2] Create `.github/workflows/release.yml` with the header, `on: push: tags: ["v*"]`, top-level `permissions: contents: read`, and the `build` job on `ubuntu-24.04` implementing steps 1–8 of [contracts/release-workflow.md](./contracts/release-workflow.md): checkout with `fetch-depth: 0`; fail if `gh release view "$GITHUB_REF_NAME"` finds a release; fail unless `git merge-base --is-ancestor "$GITHUB_SHA" origin/main`; fail with both versions in the message unless the tag without `v` equals the `version` read from `Cargo.toml`; install stable Rust; `cargo test --locked`; `cargo build --release --locked`; `cargo install --locked cargo-generate-rpm@0.21.1` and `cargo generate-rpm`. Pin actions by commit SHA using the pins in `/home/iamtuominen/git/learning-corner/.github/workflows/release.yml` (`actions/checkout`, `dtolnay/rust-toolchain`) (depends on T014)
- [ ] T021 [US2] Add step 9 to the `build` job in `.github/workflows/release.yml`: run a `fedora:44` container with the generated `.rpm` mounted, `dnf install -y` it, and fail unless `local-ai --version` prints exactly `local-ai <version>` and `local-ai --help` exits `0` (depends on T020)
- [ ] T022 [US2] Add steps 10–11 to the `build` job in `.github/workflows/release.yml`: copy the package to `dist-release/local-ai-<version>.x86_64.rpm`, write `dist-release/SHA256SUMS` with `sha256sum -- * > SHA256SUMS` run inside that folder, print it, and upload `dist-release/` as the `packages` artifact with `if-no-files-found: error` using the pinned `actions/upload-artifact` (depends on T021)
- [ ] T023 [US2] Add the `draft` job to `.github/workflows/release.yml`: `needs: build`, `permissions: contents: write`, no checkout; download the `packages` artifact with the pinned `actions/download-artifact`, then `gh release create "$GITHUB_REF_NAME" dist-release/* --repo "$GITHUB_REPOSITORY" --draft --verify-tag --title "local-ai ${GITHUB_REF_NAME#v}" --generate-notes` with `GH_TOKEN: ${{ github.token }}`. Confirm the file parses with `python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/release.yml'))"` (depends on T022)
- [ ] T024 [P] [US2] Create `docs/release.md` with a **Cutting a release** section: bump `version` in `Cargo.toml`, merge to `main`, `git tag v<version>` and `git push origin v<version>`, wait for the draft, read it, publish with `gh release edit v<version> --draft=false`; list what stops a release (tag off `main`, version mismatch, failing tests, release already exists) and note that the workflow never publishes on its own

**Checkpoint**: The workflow file is complete and parses. It can only be exercised by pushing a tag after merge (Phase 6).

---

## Phase 5: User Story 3 - Check that a download is intact (Priority: P3)

**Goal**: A user can verify a downloaded package against the release's checksum file.

**Independent Test**: Download both release files, run `sha256sum -c SHA256SUMS`, see `OK`; alter the package and see `FAILED` — `quickstart.md` Scenario 7 (run after merge, T030). The checksum file itself is produced by T022.

### Implementation for User Story 3

- [ ] T025 [P] [US3] Add a **Verifying a download** section to `docs/release.md` (download the `.rpm` and `SHA256SUMS` into one folder, run `sha256sum -c SHA256SUMS`, expect `OK`) and link to it from the Install section of `README.md` (depends on T017, T024)

**Checkpoint**: The documentation covers install, verify, and release (FR-019).

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Project documentation obligations from `plan.md`'s Constitution Check, quality gates, and the validation that can only happen once the workflow is on `main`.

- [ ] T026 [P] Update `CLAUDE.md`: the binary is `local-ai` (Project Intent line, the `cargo run` examples and the `AI_WORKSPACE_HOME=/tmp/ai-test cargo run -- init` note stay valid), mention `COMMAND_NAME` in `lib.rs` as the single source for the command name, note `.github/workflows/release.yml` and `docs/release.md`, and change the "Current shipped versions" line to end with `→ v0.6 (Release packaging, current Cargo.toml version)`
- [ ] T027 [P] Update `ROADMAP.md` so the v0.6 entry is "Release packaging" (Fedora `.rpm` from a version tag; command renamed to `local-ai`) and remove the TUI entry from the v0.6 slot, leaving a one-line note that the next user-interface step (TUI or desktop app) is undecided; add `ADR-0006` to `DECISIONS.md` in the existing format recording the three decisions and their reasons: packaging with `cargo-generate-rpm` via a tag-triggered draft release, the command name `local-ai` from one constant, and the `MIT OR Apache-2.0` license
- [ ] T028 Run `cargo fmt`, `cargo clippy --all-targets` and `cargo test --locked`; fix every warning and failure (depends on all implementation tasks)
- [ ] T029 (maintainer) After the pull request is merged to `main`: run `quickstart.md` Scenario 5 (a mismatched tag and an off-`main` tag each fail before compiling and create no release; delete the test tags afterwards), then Scenario 6 (push `v0.6.0`, confirm a draft with `local-ai-0.6.0.x86_64.rpm` and `SHA256SUMS` appears within 15 minutes). These push tags to GitHub — do them only with the maintainer's go-ahead (depends on T023, merge)
- [ ] T030 (maintainer) Run `quickstart.md` Scenario 7 against the `v0.6.0` draft (`sha256sum -c` passes, then fails on an altered copy); if all scenarios passed, the maintainer publishes with `gh release edit v0.6.0 --draft=false`, installs the released package, and deletes the old `~/.local/bin/ai` (depends on T029)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies.
- **Foundational (Phase 2)**: depends on Setup. Blocks every user story.
- **US1 (Phase 3)**: depends on Foundational.
- **US2 (Phase 4)**: depends on Foundational and on T014 (the package metadata the workflow packs).
- **US3 (Phase 5)**: depends on T017 and T024 for the documents it extends; its artifact comes from T022.
- **Polish (Phase 6)**: T026–T028 after implementation; T029–T030 after the pull request is merged.

### Within Each Phase

- Foundational: T004–T005 (tests) before T008–T012; T006–T007 before T008–T012; T013 last.
- US1: T014 → T015 → T016. T017 and T018 any time after Foundational.
- US2: T020 → T021 → T022 → T023, all in one file. T019 and T024 are independent of them.

### Parallel Opportunities

- T002 alongside T001.
- T005 alongside T004 (different files).
- T009, T010, T011, T012 together once T007 is done (four different files); T008 alongside them.
- T017, T018, T024 together (different documents).
- T026 and T027 together.

---

## Parallel Example: Foundational

```bash
# After T006 and T007:
Task: "T008 app.rs — help, usage errors, version line from COMMAND_NAME"
Task: "T009 error.rs — search usage message"
Task: "T010 session.rs — gate labels and rerun hint"
Task: "T011 ui.rs — validation-gate lines and no-results hint"
Task: "T012 workspace.rs — suggested_fix strings"
# Then T013 to confirm nothing still says `ai`.
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: version and license.
2. Phase 2: the rename, with its tests.
3. Phase 3: package metadata, local build, install check.
4. **Stop and validate**: a locally built `.rpm` installs `local-ai` on Fedora. The maintainer can already replace the July hand-copied binary at this point.

### Incremental Delivery

1. Setup + Foundational → `local-ai` builds and passes tests.
2. US1 → installable package (MVP).
3. US2 → the workflow that builds it from a tag.
4. US3 → verification documented.
5. Polish → docs, ADR, quality gates; merge; tag `v0.6.0`; publish.

### Notes

- One pull request by the maintainer's decision (plan.md Complexity Tracking). Keep the commits separable: license and version; rename; package metadata; workflow; documentation.
- `specs/001-*` to `specs/006-*` are historical and keep the `ai` name.
- Tasks marked (maintainer) change the machine (`sudo dnf`) or publish to GitHub (tags, release). Do not run them without the maintainer's explicit go-ahead.
- The workflow cannot be tested before it is on `main`; a failure in T029 is fixed with a follow-up pull request, after which the tag is deleted and pushed again on the new `main`.
