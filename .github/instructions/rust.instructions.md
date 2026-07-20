---
description: "Use when writing or reviewing Rust source files for the workspace core, project manager, runtime manager, search, permissions, context, or UI layers. Covers layering, errors, and module boundaries."
applyTo: "src/**/*.rs"
---
# Rust Guidelines

- Keep Rust code aligned with the layered architecture in `PROJECR_STRUCTURE.md` and `ARCHITECTURE.md`.
- Keep UI code out of core logic.
- Keep runtime, permissions, context, search, and project concerns in separate modules.
- Prefer explicit error handling with `Result` over panics for recoverable failures.
- Avoid `unwrap()` and `expect()` outside of tests unless the failure is truly impossible.
- Favor small, replaceable components over large monoliths.
- Keep filesystem access in the project and workspace layers, not in UI or high-level orchestration code.