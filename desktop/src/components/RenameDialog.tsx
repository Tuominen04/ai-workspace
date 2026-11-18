import { useCallback, useEffect, useState, type FormEvent } from "react";

import { engine, type ProjectRenamed } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { useLoad } from "../lib/useLoad";
import { Failure } from "./Failure";

type Props = {
  /** The project's folder name. */
  name: string;
  onRenamed: (renamed: ProjectRenamed) => void;
  onCancel: () => void;
};

/**
 * Renaming a project (ADR-0015 point 7): the new name becomes the display name and, normalized,
 * the folder ("Home Security" → home-security). Prefilled with the current display name.
 */
export function RenameDialog({ name, onRenamed, onCancel }: Props) {
  const [details] = useLoad(() => engine.project.show(name), name);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onCancel();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onCancel]);

  return (
    <div className="overlay">
      <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="rename-title">
        <h2 id="rename-title">Rename {name}</h2>
        {details.state === "loading" ? (
          <p className="muted">Reading the project…</p>
        ) : (
          <RenameForm
            name={name}
            current={details.state === "ready" ? details.value.display_name : name}
            onRenamed={onRenamed}
            onCancel={onCancel}
          />
        )}
      </div>
    </div>
  );
}

function RenameForm({
  name,
  current,
  onRenamed,
  onCancel,
}: {
  name: string;
  current: string;
  onRenamed: (renamed: ProjectRenamed) => void;
  onCancel: () => void;
}) {
  const [value, setValue] = useState(current);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureInfo | null>(null);
  const dismiss = useCallback(() => setFailure(null), []);
  const folder = folderFor(value);
  const unchanged = value.trim() === current.trim();

  async function save(event: FormEvent) {
    event.preventDefault();
    if (value.trim() === "" || unchanged) return;
    setBusy(true);
    setFailure(null);
    try {
      onRenamed(await engine.project.rename(name, value));
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form className="model-form" onSubmit={save}>
      <input
        value={value}
        onChange={(event) => setValue(event.target.value)}
        aria-label="New name"
        autoFocus
        onFocus={(event) => event.target.select()}
      />
      <p className="muted small">
        {folder === null
          ? "Letters, digits, spaces, - and _ only."
          : folder === name
            ? "The folder stays the same; only the shown name changes."
            : `The folder becomes Projects/${folder}. An open chat in it is ended and its temporary grants end.`}
      </p>
      <div className="row">
        <button type="submit" disabled={busy || value.trim() === "" || unchanged}>
          {busy ? "Renaming…" : "Rename"}
        </button>
        <button type="button" className="quiet" onClick={onCancel} disabled={busy}>
          Cancel
        </button>
      </div>
      {failure && <Failure message={failure.message} kind={failure.kind} onDismiss={dismiss} />}
    </form>
  );
}

/**
 * The folder a name becomes, as the core normalizes it (lowercase, spaces to hyphens), or
 * `null` for characters the core refuses. Only a preview: the core decides.
 */
function folderFor(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed === "" || !/^[A-Za-z0-9 _-]+$/.test(trimmed)) return null;
  return trimmed.toLowerCase().split(/\s+/).join("-");
}
