# Contract: Release Workflow

Covers FR-009 to FR-017 (see spec.md). File: `.github/workflows/release.yml`. Modelled on Learning Corner's `release.yml`.

## Trigger

```yaml
on:
  push:
    tags: ["v*"]
```

Nothing else starts it. Tags that do not begin with `v` are ignored.

## Permissions

Workflow default: `contents: read`. Only the `draft` job raises it, to `contents: write`.

## Job 1: `build` (`contents: read`, `ubuntu-24.04`)

Steps, in order. Any failure ends the run with no release.

| # | Step | Fails when | Requirement |
| --- | --- | --- | --- |
| 1 | Check out with full history | — | — |
| 2 | Release must not exist for this tag | a release (draft or published) already exists | FR-016 |
| 3 | Tag is on `main` | the tagged commit is not an ancestor of `origin/main` | FR-010 |
| 4 | Tag matches the version | tag without `v` ≠ `version` in `Cargo.toml`; the message states both | FR-011 |
| 5 | Install the stable Rust toolchain | — | — |
| 6 | `cargo test --locked` | any test fails, or the lock is out of date | FR-012, FR-013 |
| 7 | `cargo build --release --locked` | build fails | FR-013 |
| 8 | Install `cargo-generate-rpm` at the pinned version, run it | packaging fails | FR-001 |
| 9 | Smoke test: install the `.rpm` in a `fedora:44` container, run `local-ai --version` and `local-ai --help` | install fails, the command is missing, or the version line is not `local-ai <version>` | FR-002, FR-003, FR-005 |
| 10 | Collect: copy to `local-ai-<version>.x86_64.rpm`, write `SHA256SUMS` | — | FR-014 |
| 11 | Upload the folder as the `packages` artifact | no files found | — |

Steps 2 to 4 run before any compilation.

## Job 2: `draft` (`contents: write`, needs `build`)

| # | Step |
| --- | --- |
| 1 | Download the `packages` artifact |
| 2 | `gh release create "$TAG" <files> --draft --verify-tag --title "local-ai <version>" --generate-notes` |

This job checks out no code and runs no project or dependency code (FR-017). It creates a **draft**; it never publishes (FR-015).

## Outputs

On success, one draft release for the tag with exactly two assets:

```text
local-ai-<version>.x86_64.rpm
SHA256SUMS
```

## Pinning

- Third-party actions are referenced by full commit SHA with the version in a trailing comment, using the pins already reviewed in Learning Corner's workflow.
- `cargo-generate-rpm` is installed at an exact version with `--locked`.
- The Fedora container image is referenced by its release tag (`fedora:44`).

## Not included (private repository)

Build provenance attestation and tag protection rules. See research.md Decision 6.

## Maintainer procedure

```bash
# on main, with the version already bumped and merged
git tag v0.6.0
git push origin v0.6.0
# wait for the draft, read it, then:
gh release edit v0.6.0 --draft=false
```

Documented for users in `docs/release.md`.
