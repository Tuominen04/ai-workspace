import { useState } from "react";

import { engine } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { Failure } from "./Failure";

/**
 * Initializes the workspace: creates the standard layout and, if missing, the default
 * configuration. Safe to repeat; an existing configuration is kept.
 */
export function InitButton({ onDone, label = "Initialize the workspace" }: { onDone: () => void; label?: string }) {
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  async function run() {
    setBusy(true);
    setFailure(null);
    try {
      await engine.workspace.init();
      onDone();
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <button type="button" onClick={run} disabled={busy}>
        {busy ? "Initializing…" : label}
      </button>
      {failure && <Failure message={failure.message} kind={failure.kind} />}
    </>
  );
}
