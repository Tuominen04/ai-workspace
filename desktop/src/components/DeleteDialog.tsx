import { useEffect, useRef, useState } from "react";

import { engine, type ProjectDeleted } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { Failure } from "./Failure";

type Props = {
  name: string;
  /** Whether it is an archived project. */
  archived: boolean;
  onDeleted: (deleted: ProjectDeleted) => void;
  onCancel: () => void;
};

/**
 * Deleting a project (ADR-0015): first a confirmation, then the folder moves to the desktop
 * Trash. Only if the Trash cannot take it does a second warning offer to erase it for good.
 */
export function DeleteDialog({ name, archived, onDeleted, onCancel }: Props) {
  // "trash": the first confirmation; "erase": the Trash refused, offer erasing.
  const [step, setStep] = useState<"trash" | "erase">("trash");
  const [reason, setReason] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureInfo | null>(null);
  const cancel = useRef<HTMLButtonElement>(null);

  // The safe choice has the focus, and Escape cancels.
  useEffect(() => {
    cancel.current?.focus();
  }, [step]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && !busy) onCancel();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [busy, onCancel]);

  async function remove(erase: boolean) {
    setBusy(true);
    setFailure(null);
    try {
      onDeleted(await engine.project.delete(name, archived, erase));
    } catch (thrown) {
      const info = failureOf(thrown);
      if (info.kind === "TrashUnavailable" && !erase) {
        setReason(info.message);
        setStep("erase");
      } else {
        setFailure(info);
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="overlay">
      <div className="dialog" role="alertdialog" aria-modal="true" aria-labelledby="delete-title">
        {step === "trash" ? (
          <>
            <h2 id="delete-title">Delete {name}?</h2>
            <p>
              The project folder, with all its files and past sessions, moves to the Trash. You can
              restore it from the Files app&apos;s Trash.
            </p>
            <div className="row">
              <button type="button" className="danger" onClick={() => void remove(false)} disabled={busy}>
                {busy ? "Moving…" : "Move to Trash"}
              </button>
              <button type="button" className="quiet" ref={cancel} onClick={onCancel} disabled={busy}>
                Cancel
              </button>
            </div>
          </>
        ) : (
          <>
            <h2 id="delete-title">Erase {name} permanently?</h2>
            <p className="failure">{reason}</p>
            <p>
              Erasing removes the folder for good: it cannot be restored. Cancel keeps the project
              as it is.
            </p>
            <div className="row">
              <button type="button" className="danger" onClick={() => void remove(true)} disabled={busy}>
                {busy ? "Erasing…" : "Erase permanently"}
              </button>
              <button type="button" className="quiet" ref={cancel} onClick={onCancel} disabled={busy}>
                Cancel
              </button>
            </div>
          </>
        )}
        {failure && <Failure message={failure.message} kind={failure.kind} />}
      </div>
    </div>
  );
}
