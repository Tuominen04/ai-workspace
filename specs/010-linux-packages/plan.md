# Implementation Plan: Linux Packages (1.1)

**Spec**: `specs/010-linux-packages/spec.md` · **Decision**: ADR-0017

The app's code does not change; the work is packaging, the release workflow and documentation. Two pull requests after this spec:

1. **Packages and release workflow**
   - `desktop/src-tauri/tauri.conf.json`: bundle targets `rpm`, `deb`, `appimage`; `linux.deb` with the same licence files as the `.rpm`, and the description without "for Fedora".
   - `.github/workflows/release.yml`: the build job collects all three files under the names in FR-001 into `dist-release/`; smoke tests for the `.deb` in `ubuntu:24.04` and `debian:13` containers (`apt install`, command, menu entry, `ldd`) and for the AppImage (`--appimage-extract`, `ldd` on the extracted binary), next to the existing Fedora one; `SHA256SUMS` and the attestation cover `dist-release/*` packages.
   - A dry run of the build and smoke tests in a pull request (a `workflow_dispatch` or a temporary job), since the release workflow only runs on tags.
2. **Documentation**
   - README: install per package and the supported systems (FR-005), including `libfuse2t64` for the AppImage on Ubuntu 24.04.
   - `docs/user-guide.md`: the same, plus removal per package.
   - `docs/release.md`: verifying each file; `docs/checklist/release-checklist.md`: the short Linux section per VM (spec, "The short checklist per VM").
   - `CLAUDE.md` and `ARCHITECTURE.md`: releases are the `.rpm`, `.deb` and AppImage.

Then version 1.1.0: the full checklist on Fedora and the short one on VMs 1 to 3, filled in as `docs/checklist/v1.1.0.md`.

## Risks

- **AppImage size and WebKitGTK inside it**: a known pain point of Tauri AppImages (graphics drivers, Wayland). VM 3 is there to catch it; if it does not work well enough, the AppImage is dropped from 1.1 and the `.deb` ships alone.
- **glibc floor**: if a dependency needs newer libraries than Ubuntu 24.04 has, VM 1 shows it; the fix is to build on an older runner, not to drop 24.04.
