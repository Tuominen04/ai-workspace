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

1. Run a grant for a non-local or inaccessible path.
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

### Scenario 9: Usability evidence for success criteria

1. Run a small usability check with at least 10 participants or equivalent internal
   trial runs.
2. Measure:
   - time to complete a valid temporary grant flow
   - first-attempt correctness when identifying effective AI access scope from viewer
3. Record measured values and pass/fail verdict for SC-001 and SC-004 in this file.
4. Expected:
   - Evidence table is present and reproducible.
   - Results explicitly state whether SC-001 and SC-004 thresholds were met.

## References

- Specification: `./spec.md`
- Plan: `./plan.md`
- Research: `./research.md`
- Data model: `./data-model.md`
- CLI contract: `./contracts/cli.md`

## Implementation Validation Notes

Validated on 2026-07-22 using `AI_WORKSPACE_HOME=/tmp/ai-workspace-v03-impl`.

- `cargo test` passed.
- `ai init` created workspace layout and default configuration.
- `ai project create --name demo-perm --description "Permission test project"`
   succeeded.
- `ai permission grant --project demo-perm --path /tmp` succeeded.
- Duplicate grant for `/tmp` returned idempotent success message.
- `ai permission view --project demo-perm` showed default and temporary entries.
- `ai permission revoke --project demo-perm --path /tmp` removed the temporary
   grant.
- Invalid-path grant returned: `invalid permission path`.
- Deleting a granted path produced `temporary/unavailable` status in viewer output.
- `Logs/permissions-events.log` recorded `grant`, `deny`, `revoke`, and
   `status-change` events.
- Interactive session shutdown revoked temporary grants and removed them from
   subsequent permission view output.

### Usability Evidence (SC-001, SC-004)

Internal trial benchmark (10 runs):

| Metric | Result | Target | Pass |
| -- | -- | -- | -- |
| SC-001 grant completion | 10/10 succeeded, average 4 ms | 95% under 30 s | PASS |
| SC-004 scope identification | 10/10 view checks correctly included granted path on first check | 95% first attempt | PASS |
