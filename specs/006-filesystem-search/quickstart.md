# Quickstart: v0.5 Filesystem Search

Validation scenarios for the feature described in [spec.md](./spec.md), using the contracts in [contracts/](./contracts/) and entities in [data-model.md](./data-model.md).

## Prerequisites

```bash
cargo build
export AI_WORKSPACE_HOME=/tmp/ai-search-quickstart
rm -rf "$AI_WORKSPACE_HOME"
cargo run -- init
```

Create a project with enough Markdown content that search has something to differentiate:

```bash
cargo run -- project create --name demo --description "search quickstart"
mkdir -p "$AI_WORKSPACE_HOME/Projects/demo/Knowledge"
cat > "$AI_WORKSPACE_HOME/Projects/demo/Knowledge/ollama-setup.md" <<'EOF'
# Ollama Setup
Install Ollama locally and start the runtime before running ai.
EOF
cat > "$AI_WORKSPACE_HOME/Projects/demo/Knowledge/permissions.md" <<'EOF'
# Permissions
Temporary permissions are granted per session and revoked on exit.
EOF
```

## Scenario 1 — Standalone search finds relevant content (User Story 1 & 3)

```bash
cargo run -- search --query "ollama runtime" --project demo
```

**Expected**: `ollama-setup.md` is returned (and ranked above `permissions.md`, which doesn't mention Ollama), with a snippet showing the matched text — per `contracts/cli-search.md`'s success output. Confirms FR-001, FR-003, FR-005.

## Scenario 2 — No matches reported clearly (edge case)

```bash
cargo run -- search --query "kubernetes cluster autoscaling" --project demo
```

**Expected**: Exit code `0`, "No documents matched" message — not an error. Confirms FR-004.

## Scenario 3 — Permission boundary is respected (SC-005)

```bash
mkdir -p /tmp/ai-search-outside
echo "# Secret\nunrelated ollama content" > /tmp/ai-search-outside/secret.md
cargo run -- search --query "ollama" --project demo
```

**Expected**: `secret.md` never appears in results — it was never collected as a candidate because it isn't under Base or the `demo` project's granted paths. Confirms FR-002/FR-008.

Now grant it and confirm it becomes visible:

```bash
# (interactive session required to grant; see ai permission grant --path /tmp/ai-search-outside)
```

## Scenario 4 — Context assembly uses relevance, not just file order (User Story 2)

Populate enough Knowledge files to exceed `DEFAULT_MAX_CONTEXT_SOURCES` (8), where only a couple mention a specific topic, then start an interactive session and ask a targeted question:

```bash
cargo run -- 
# select "demo" project
# at the chat prompt: "How do I set up Ollama?"
```

**Expected**: The system prompt sent to the runtime for that turn (visible via `ai`'s session summary / chat context, or by temporarily logging `context.assembled_content` during manual verification) includes `ollama-setup.md`, even if it wasn't among the first 8 files in priority order. Confirms FR-006/FR-007 and User Story 2's acceptance scenarios.

## Scenario 5 — Small workspace still gets everything (User Story 2, AC2)

In a workspace with only 2-3 Knowledge files (well under the caps), start a session and confirm the initial context (before any question is asked, `query = None` per `contracts/context-builder-integration.md`) still includes all of them — unchanged from v0.4 behavior.

## Automated coverage

These scenarios map to `tests/search_tests.rs` (Scenarios 1–3) and `tests/context_tests.rs` (Scenarios 4–5), run via:

```bash
cargo test --test search_tests
cargo test --test context_tests
```
