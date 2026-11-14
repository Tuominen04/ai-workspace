# Research: Release Packaging

No `NEEDS CLARIFICATION` items remained in the Technical Context. The decisions below settle the choices the spec left to planning.

## Decision 1: Build the package with `cargo-generate-rpm`

- **Decision**: Use `cargo-generate-rpm`, pinned to `0.21.1`, with the package description under `[package.metadata.generate-rpm]` in `Cargo.toml`.
- **Rationale**: It builds an `.rpm` directly from the release binary with no `rpmbuild`, no `.spec` file, and no Fedora build container, so it runs on the same `ubuntu-24.04` runner Learning Corner uses. The metadata lives in TOML next to the version it must match. It is a build-time tool only; nothing is added to the shipped binary.
- **Alternatives considered**:
  - *`rpmbuild` with a hand-written `.spec` in a Fedora container* — the most Fedora-native route, but adds a second file format, a container build, and a second place the version is written.
  - *Tauri's bundler (what Learning Corner uses)* — only applies to a Tauri app. If a desktop UI is built later it will bring its own bundler; this decision does not block that.
  - *`cargo-rpm`* — unmaintained.

## Decision 2: Build on `ubuntu-24.04`, verify on Fedora

- **Decision**: Compile and package on the `ubuntu-24.04` runner; then install the resulting `.rpm` in a `fedora:44` container inside the same build job and run `ai-workspace --version` and `ai-workspace --help`.
- **Rationale**: The binary links only the C library (`ureq` is built without TLS), and a binary built against Ubuntu 24.04's older C library runs on Fedora 44's newer one. The container step turns that reasoning into a check on every release, and it also proves FR-002, FR-003, and FR-005 (command on the path, named `ai-workspace`, version matches) before a draft is created.
- **Alternatives considered**: Building inside a Fedora container (slower, no added safety once the smoke test exists); trusting the build without installing it (leaves FR-002/003/005 unverified until a human installs the package).

## Decision 3: Package dependencies are detected, the AI runtime is not required

- **Decision**: Let `cargo-generate-rpm` derive library requirements from the binary with its built-in detection (`cargo generate-rpm --auto-req builtin`; the metadata key only accepts `"no"`), and set `require-sh = false` because the package runs no scripts. Do not add a requirement on Ollama.
- **Rationale**: Built-in detection needs no RPM tooling on the runner and records the C-library requirement, so `dnf` refuses the package on a system too old to run it. Ollama is not in Fedora's repositories; requiring it would make the package uninstallable. The tool already reports a missing runtime at session start.
- **Alternatives considered**: `auto-req = "no"` (package would install on systems where the binary cannot start); a weak dependency on Ollama (nothing to point it at).

## Decision 4: One constant for the command name

- **Decision**: Add `pub const COMMAND_NAME: &str = "ai-workspace";` to `src/lib.rs` and build every usage, help, hint, and version string from it. The Cargo binary target is renamed to `ai-workspace`; the crate (`ai-workspace`, library `ai_workspace`) keeps its name.
- **Rationale**: The name `ai` is written out about 45 times across `app.rs`, `error.rs`, `session.rs`, `ui.rs`, and `workspace.rs`. A plain find-and-replace would leave 45 copies of the new name to miss next time. One constant plus one test (`--help` output names `ai-workspace` and contains no `ai ` usage line) makes the rename checkable. Keeping the crate name avoids touching every `use ai_workspace::...` in the tests for no user-visible gain.
- **Alternatives considered**: Literal replacement (no guard against a missed string); reading the name from `env!("CARGO_BIN_NAME")` (only defined when compiling the binary target, not the library where the strings live); renaming the crate too (large diff, no benefit).

## Decision 5: Version `0.6.0`

- **Decision**: This feature ships as `0.6.0`. `ROADMAP.md`'s v0.6 entry becomes release packaging; the former v0.6 TUI entry is removed from that slot.
- **Rationale**: The command name changes, which is a user-visible break, and the release workflow needs a version that has never been tagged. The maintainer dropped the TUI draft on 2026-11-14 in favour of packaging first. Where a user interface goes on the roadmap afterwards (TUI or a desktop app) is the maintainer's call and is left open in `ROADMAP.md`, not decided here.
- **Alternatives considered**: `0.5.1` (understates a renamed command); `1.0.0` (premature).

## Decision 6: Mirror Learning Corner's release workflow, minus what a private repository cannot do

- **Decision**: One workflow, `.github/workflows/release.yml`, triggered by `v*` tags, with two jobs:
  - **build** (`contents: read`): confirm no release exists for the tag; confirm the tag is on `main` and equals the `Cargo.toml` version; `cargo test --locked`; `cargo build --release --locked`; package; smoke-test in `fedora:44`; name the file `ai-workspace-<version>.x86_64.rpm`; write `SHA256SUMS`; upload as an artifact.
  - **draft** (`contents: write`): download the artifact and run `gh release create --draft --verify-tag --generate-notes`. It checks out nothing and runs no project code.
  Third-party actions are pinned by commit SHA, reusing the exact pins already reviewed in Learning Corner's workflow.
- **Rationale**: The maintainer already knows this flow. The two-job split is what satisfies FR-017. `gh release create` fails when a release for the tag exists (FR-016); the early check in the build job just fails faster.
- **Left out, and why**: build provenance attestation and a tag ruleset — neither is available on a private repository on the current plan. `.deb` and AppImage — out of scope (Fedora only). Each can be added later without restructuring the workflow.
- **Alternatives considered**: A single job with write permission (violates FR-017); publishing immediately instead of drafting (the maintainer wants to read the draft first).

## Decision 7: License files and where they install

- **Decision**: Add `LICENSE-MIT` and `LICENSE-APACHE` at the repository root with the same texts and copyright holder as Learning Corner; set `license = "MIT OR Apache-2.0"` in `Cargo.toml`; install both texts to `/usr/share/licenses/ai-workspace/`; add a License section to `README.md`.
- **Rationale**: Matches the maintainer's other project and the Rust ecosystem's convention. `/usr/share/licenses/<package>/` is where Fedora packages put license texts, though `cargo-generate-rpm` 0.21.1 cannot flag them as license files, so `rpm -qL` does not list them (found during implementation).
- **Alternatives considered**: A single combined `LICENSE` file (diverges from Learning Corner for no benefit).

## Decision 8: Tests must pass on a runner without Ollama

- **Decision**: Treat "the suite passes on a clean runner" as a requirement to confirm during implementation, by running `cargo test --locked` with `ollama` absent from `PATH` before the first tag is pushed.
- **Rationale**: FR-012 makes a failing test block the release, and the runner has no Ollama. The runtime tests talk to a mock server on a local port rather than a real runtime, so they are expected to pass; this has only been observed on a machine that has Ollama installed.
- **Alternatives considered**: Installing Ollama on the runner (slow, and tests should not need it).

## Decision 9: Documentation scope

- **Decision**: Update the documents that describe current behavior — `README.md`, `CLAUDE.md`, `docs/workspace.md`, `docs/permissions.md`, `ROADMAP.md` — and add `docs/release.md` and ADR-0006 in `DECISIONS.md`. Leave `specs/001-*` to `specs/006-*` untouched.
- **Rationale**: FR-004 and FR-019 concern what a user reads today. Earlier specs are records of what was decided at the time; rewriting them would falsify the history.
- **Note for the docs**: the workspace is still found through `AI_WORKSPACE_HOME` (name unchanged) or `~/AI`. An old hand-copied `~/.local/bin/ai` is not removed by the package and should be deleted by hand.
