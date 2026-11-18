import { useState, type FormEvent } from "react";

import { engine, type ProjectDetails, type ProjectMoved, type WorkspaceStatus } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { useLoad } from "../lib/useLoad";
import { counted, dateText } from "../lib/words";
import { Failure } from "./Failure";

type Props = {
  status: WorkspaceStatus;
  project: string | null;
  /** The selected archived project, if one is. */
  archived: string | null;
  onArchived: (moved: ProjectMoved) => void;
  onRestored: (moved: ProjectMoved) => void;
  onCloned: (clone: ProjectDetails) => void;
};

/** The selected project's details, an archived project, or the workspace's folders. */
export function Overview({ status, project, archived, onArchived, onRestored, onCloned }: Props) {
  if (archived !== null) {
    const summary = status.archived.find((p) => p.name === archived);
    return <ArchivedOverview name={archived} path={summary?.path ?? ""} onRestored={onRestored} />;
  }
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
  return <ProjectOverview name={project} onArchived={onArchived} onCloned={onCloned} />;
}

/** Runs one project action (archive, restore, clone) and shows why it was refused, if it was. */
function useProjectAction<T>(run: () => Promise<T>, done: (result: T) => void) {
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureInfo | null>(null);
  async function act() {
    setBusy(true);
    setFailure(null);
    try {
      done(await run());
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }
  return { busy, failure, act };
}

/** An archived project: read-only, with Restore (ADR-0015). */
function ArchivedOverview({
  name,
  path,
  onRestored,
}: {
  name: string;
  path: string;
  onRestored: (moved: ProjectMoved) => void;
}) {
  const restore = useProjectAction(() => engine.project.restore(name), onRestored);
  return (
    <section className="panel">
      <div className="panel-head">
        <h2>{name}</h2>
        <button type="button" onClick={() => void restore.act()} disabled={restore.busy}>
          {restore.busy ? "Restoring…" : "Restore"}
        </button>
      </div>
      <p className="muted">
        Archived. Restore it to chat in it, search it or change its permissions again.
      </p>
      <dl className="facts">
        <dt>Folder</dt>
        <dd className="path">{path}</dd>
      </dl>
      {restore.failure && <Failure message={restore.failure.message} kind={restore.failure.kind} />}
    </section>
  );
}

function ProjectOverview({
  name,
  onArchived,
  onCloned,
}: {
  name: string;
  onArchived: (moved: ProjectMoved) => void;
  onCloned: (clone: ProjectDetails) => void;
}) {
  const archive = useProjectAction(() => engine.project.archive(name), onArchived);
  const clone = useProjectAction(() => engine.project.clone(name), onCloned);
  const [details, reload] = useLoad(() => engine.project.show(name), name);

  if (details.state === "loading") return <p className="muted panel">Loading {name}…</p>;
  const archiveButton = (
    <button
      type="button"
      className="quiet"
      onClick={() => void archive.act()}
      disabled={archive.busy}
      title="Move it to Projects/Archive/; it can be restored"
    >
      {archive.busy ? "Archiving…" : "Archive"}
    </button>
  );
  // Archiving does not read the project, so it is offered even when its details cannot load.
  if (details.state === "failed")
    return (
      <section className="panel">
        <div className="panel-head">
          <h2>{name}</h2>
          {archiveButton}
        </div>
        <Failure message={details.message} kind={details.kind} />
        {archive.failure && <Failure message={archive.failure.message} kind={archive.failure.kind} />}
      </section>
    );

  const p = details.value;
  return (
    <section className="panel">
      <div className="panel-head">
        <h2>{p.display_name}</h2>
        <div className="row">
          <button
            type="button"
            className="quiet"
            onClick={() => void clone.act()}
            disabled={clone.busy}
            title={`Copy it, files and past sessions, into ${name}-clone`}
          >
            {clone.busy ? "Cloning…" : "Clone"}
          </button>
          {archiveButton}
        </div>
      </div>
      {archive.failure && <Failure message={archive.failure.message} kind={archive.failure.kind} />}
      {clone.failure && <Failure message={clone.failure.message} kind={clone.failure.kind} />}
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
