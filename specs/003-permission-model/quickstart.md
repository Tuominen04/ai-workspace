# Quickstart Validation: v0.3 Permission Model

## Prerequisites

- Fedora-compatible Linux environment.
- Rust toolchain installed (`cargo`, `rustc`).
- Active repository checkout with v0.3 permission-model artifacts.

## Setup

1. Build the binary:
   - `cargo build`
2. Initialize workspace if needed:
   - `./target/debug/ai init`
3. Start from a valid project context:
   - `./target/debug/ai validate`

## Validation Scenarios

### Scenario 1: Grant a valid temporary permission

1. Run:
   - `./target/debug/ai permission grant --path ~/Downloads/api.pdf`
2. Expected:
   - Grant is accepted.
   - Permission appears in the viewer as temporary and active.

### Scenario 2: Duplicate grant is idempotent

1. Run the same grant command twice for the same path.
2. Expected:
   - Second invocation succeeds without creating a duplicate active entry.
   - Viewer still shows one entry for that canonical path.

### Scenario 3: Deny out-of-bound path

1. Run a grant for a path outside approved local roots.
2. Expected:
   - Command fails with actionable boundary-violation output.
   - Path does not appear as an active permission.

### Scenario 4: Inspect large permission set

1. Create enough temporary grants to exceed one page.
2. Run:
   - `./target/debug/ai permission view --page 1 --page-size 25`
   - `./target/debug/ai permission view --page 2 --page-size 25`
   - `./target/debug/ai permission view --filter api`
3. Expected:
   - Results are deterministically sorted.
   - Paging and filtering return consistent, complete subsets.

### Scenario 5: Unavailable path behavior

1. Grant a valid path.
2. Make the path temporarily unreachable.
3. Run:
   - `./target/debug/ai permission view`
4. Expected:
   - Grant remains listed but marked unavailable.
   - Access attempts to that path are blocked until path is reachable again.

### Scenario 6: Session-end cleanup

1. Grant at least one temporary path.
2. End the session and start a new one for the same project.
3. Run:
   - `./target/debug/ai permission view`
4. Expected:
   - Previous session temporary grants are absent.
   - Default grants remain present.

### Scenario 7: Project-isolation guard

1. In project A session, create temporary grants.
2. Start session for project B.
3. Run:
   - `./target/debug/ai permission view`
4. Expected:
   - Project A temporary grants are not active in project B session.

### Scenario 8: Permission event logging

1. Perform one successful grant, one denied grant, one revoke, and one unavailable
   status transition.
2. Inspect local structured logs.
3. Expected:
   - Each action emits exactly one corresponding event.
   - Events include path, session context, event type, and timestamp fields.

## References

- Specification: `./spec.md`
- Plan: `./plan.md`
- Research: `./research.md`
- Data model: `./data-model.md`
- CLI contract: `./contracts/cli.md`
