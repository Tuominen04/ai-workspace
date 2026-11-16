import { useState, type FormEvent } from "react";

import { engine, type WorkspaceStatus } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { useLoad } from "../lib/useLoad";
import { counted, dateText } from "../lib/words";
import { Failure } from "./Failure";

/** The selected project's details, or the workspace's folders when none is selected. */
export function Overview({ status, project }: { status: WorkspaceStatus; project: string | null }) {
  if (project === null) {
    return (
      <section className="panel">
        <h2>Workspace</h2>
        <dl className="facts">
          <dt>Folder</dt>
          <dd className="path">{status.root}</dd>
          <dt>Chosen by</dt>
          <dd>{status.root_source}</dd>
          <dt>Base knowledge</dt>
          <dd className="path">{status.base}</dd>
          <dt>Projects folder</dt>
          <dd className="path">{status.projects_dir}</dd>
          <dt>Projects</dt>
          <dd>{counted(status.projects.length, "project")}</dd>
        </dl>
      </section>
    );
  }
  return <ProjectOverview name={project} />;
}

function ProjectOverview({ name }: { name: string }) {
  const [details, reload] = useLoad(() => engine.project.show(name), name);

  if (details.state === "loading") return <p className="muted panel">Loading {name}…</p>;
  if (details.state === "failed")
    return (
      <section className="panel">
        <h2>{name}</h2>
        <Failure message={details.message} kind={details.kind} />
      </section>
    );

  const p = details.value;
  return (
    <section className="panel">
      <h2>{p.display_name}</h2>
      <p className="description">{p.description || <span className="muted">No description.</span>}</p>
      <dl className="facts">
        <dt>Folder</dt>
        <dd className="path">{p.path}</dd>
        <dt>Model</dt>
        <dd>
          <ModelSetting project={p.name} model={p.model} onChanged={reload} />
        </dd>
        <dt>Created</dt>
        <dd>{dateText(p.created_at)}</dd>
        <dt>Knowledge</dt>
        <dd>{p.knowledge_dir}</dd>
        <dt>Specs</dt>
        <dd>{p.specs_dir}</dd>
        <dt>Resources</dt>
        <dd>{p.resources_dir}</dd>
        <dt>Session</dt>
        <dd>{p.session_dir}</dd>
      </dl>
    </section>
  );
}

/**
 * The project's model: shown, and changeable. Installed Ollama models are offered as
 * suggestions; any name can be typed. Clearing returns to the workspace default.
 */
function ModelSetting({
  project,
  model,
  onChanged,
}: {
  project: string;
  model: string | null;
  onChanged: () => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(model ?? "");
  const [installed, setInstalled] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  async function startEditing() {
    setDraft(model ?? "");
    setFailure(null);
    setEditing(true);
    try {
      setInstalled(await engine.models.installed());
    } catch {
      setInstalled([]);
    }
  }

  async function save(next: string | null) {
    setBusy(true);
    setFailure(null);
    try {
      await engine.project.setModel(project, next);
      setEditing(false);
      onChanged();
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  if (!editing) {
    return (
      <span className="row">
        {model ?? <span className="muted">workspace default</span>}
        <button type="button" className="quiet small-button" onClick={startEditing}>
          Change
        </button>
      </span>
    );
  }

  return (
    <form
      className="model-form"
      aria-label="Project model"
      onSubmit={(event: FormEvent) => {
        event.preventDefault();
        save(draft.trim() === "" ? null : draft);
      }}
    >
      <div className="row">
        <input
          value={draft}
          onChange={(event) => setDraft(event.target.value)}
          list="installed-models"
          placeholder="Ollama model, e.g. sec"
          aria-label="Model name"
          autoFocus
        />
        <datalist id="installed-models">
          {installed.map((name) => (
            <option key={name} value={name} />
          ))}
        </datalist>
        <button type="submit" disabled={busy}>
          Save
        </button>
        <button type="button" className="quiet" disabled={busy} onClick={() => save(null)}>
          Use default
        </button>
        <button type="button" className="quiet" disabled={busy} onClick={() => setEditing(false)}>
          Cancel
        </button>
      </div>
      <p className="muted small">
        {installed.length > 0
          ? `Installed in Ollama: ${installed.join(", ")}.`
          : "Ollama did not list any models (is it running?). You can still type a name."}{" "}
        A model with its own instructions keeps them in every chat.
      </p>
      {failure && <Failure message={failure.message} kind={failure.kind} />}
    </form>
  );
}
