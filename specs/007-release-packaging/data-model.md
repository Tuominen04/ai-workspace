# Data Model: Release Packaging

Nothing in this feature is stored by the tool at run time. The entities below are release-time artifacts and the rules that tie them together.

## Project version

- **Source**: `version` in `Cargo.toml` (single source of truth).
- **Used by**: the command's `--version` output, the package version, the release title, the package file name.
- **Rule**: must be a version that has not been released before.

## Version tag

- **Form**: `v` + project version, for example `v0.6.0`.
- **Rules**:
  - Must point at a commit that is an ancestor of `origin/main` (FR-010).
  - The part after `v` must equal the project version at that commit (FR-011).
  - Must not already have a release (FR-016).
- **Relationship**: one tag → one release process run → at most one draft release.

## Package

| Field | Value | Source |
|-------|-------|--------|
| Name | `ai-workspace` | `[package.metadata.generate-rpm]` |
| Version | project version | `Cargo.toml` `version` |
| Release | `1` | tool default |
| Architecture | `x86_64` | build target |
| Summary | "Local-first AI workspace manager for Fedora" | `Cargo.toml` `description` |
| License | `MIT OR Apache-2.0` | `Cargo.toml` `license` |
| Requires | C library, detected from the binary | built-in detection |
| Install scripts | none | — |

**Installed files** (the complete list):

| Path | Mode | Kind |
|------|------|------|
| `/usr/bin/ai-workspace` | 755 | command |
| `/usr/share/licenses/ai-workspace/LICENSE-MIT` | 644 | license |
| `/usr/share/licenses/ai-workspace/LICENSE-APACHE` | 644 | license |

**Rules**: no file under any home directory; no configuration file; no service unit; no script that runs at install, upgrade, or removal (FR-008, FR-018).

## Release assets

| File | Name | Content |
|------|------|---------|
| Package file | `ai-workspace-<version>.x86_64.rpm` | the package above |
| Checksum file | `SHA256SUMS` | one SHA-256 line per other attached file |

**Rule**: every attached file except `SHA256SUMS` itself has a line in `SHA256SUMS` (FR-014).

## Draft release

- **Title**: `ai-workspace <version>`.
- **Tag**: the version tag.
- **Notes**: generated from merged changes since the previous release.
- **State transitions**:

```text
tag pushed ──► checks pass ──► package built and smoke-tested ──► Draft ──(maintainer publishes)──► Published
     │              │                       │
     └── any check, test, build, or smoke-test failure ──► no release, nothing attached
```

- **Rule**: only the maintainer moves a release from Draft to Published; the process never does (FR-015).

## Command name

- **Value**: `ai-workspace`, defined once as `COMMAND_NAME`.
- **Appears in**: help text, usage errors, validation hints, the version line, the binary file name, the package name.
- **Rule**: no user-facing output of the current version refers to the command as `ai` (FR-004).
