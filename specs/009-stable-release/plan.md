# Implementation Plan: Stable Release (1.0)

**Branch**: `009-stable-release` | **Date**: 2026-11-19 | **Spec**: [spec.md](./spec.md)

## Summary

Three pieces of work and a release gate (ADR-0016): an Ollama-in-Podman launcher, a written and tested 1.0 workspace format, and a real-Ollama release checklist; then fix what the checklist finds, and bump to 1.0.0.

## Design

### Podman launcher (`config.rs`, `runtime.rs`)

- `WorkspaceConfig` gains `runtime: Runtime` (`Ollama` | `Podman`, default `Ollama`) and `podman: PodmanSettings { image, gpu }`, all `#[serde(default)]`, so existing files load unchanged.
- `RuntimeManager::ensure_runtime(&config)` replaces `ensure_ollama`:
  1. if `127.0.0.1:11434` answers, use it (`started_here: false`);
  2. `Ollama`: today's `ollama serve`;
  3. `Podman`: `podman image exists <image>` (else a "missing image" status carrying the `podman pull` command); remove a *stopped* container named `ai-workspace-ollama` if one is left over; `podman run -d --rm --pull=never --name ai-workspace-ollama -p 127.0.0.1:11434:11434 -v ai-workspace-ollama:/root/.ollama [gpu flags] <image>`; wait for readiness as today.
- GPU flags: `nvidia` adds `--device nvidia.com/gpu=all`; `amd` adds `--device /dev/kfd --device /dev/dri` and defaults the image to `docker.io/ollama/ollama:rocm`.
- `Drop` (and session close) stops what it started: `podman stop ai-workspace-ollama` (the `--rm` removes it). The volume stays.
- Commands are built by a small pure function, so tests check the exact arguments without running Podman.
- Desktop: Configuration shows the runtime, image and GPU setting; the chat's runtime line already shows the status detail.

### 1.0 workspace format

- `docs/workspace-format.md`: every folder and file the app reads or writes, its keys and their defaults, and the compatibility rule.
- `crates/ai-workspace-core/tests/fixtures/workspace-1.0/`: a small workspace written by 1.0 (Base notes, a project with notes, a session note and permission state, an archived project, `workspace.toml` with and without the new keys, plus one unknown key).
- `format_tests.rs` copies it to a temporary folder, then loads, validates, lists, reads context and searches it.

### Release checklist

- `docs/release-checklist.md`: install the RPM on the current Fedora; for each runtime, start a chat with a real model, use every feature in the user guide, and check the files on disk. The filled-in copy goes in the 1.0 release PR.

## Pull requests

1. Ollama in Podman (spec US1).
2. 1.0 workspace format and fixture tests (US2).
3. Release checklist (US3).
4. Fixes for anything the checklist finds, one PR each.
5. Version 1.0.0, with the completed checklist attached; ROADMAP notes that v0.10 shipped as documentation only.

## Constitution check

- **Local-first**: the container is published only on `127.0.0.1`; the app never pulls images or models.
- **No hidden background services**: a container the app starts is stopped with it.
- **Filesystem as source of truth**: the format is frozen and written down.
- **Boundaries**: process work stays in `runtime.rs`, settings in `config.rs`.
