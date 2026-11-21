import { useCallback, useState, type FormEvent } from "react";

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
  onDelete: (name: string, archived: boolean) => void;
  onRename: (name: string) => void;
};

/** The selected project's details, an archived project, or the workspace's folders. */
export function Overview({
  status,
  project,
  archived,
  onArchived,
  onRestored,
  onCloned,
  onDelete,
  onRename,
}: Props) {
  if (archived !== null) {
    const summary = status.archived.find((p) => p.name === archived);
    return (
      <ArchivedOverview
        key={archived}
        name={archived}
        path={summary?.path ?? ""}
        onRestored={onRestored}
        onDelete={() => onDelete(archived, true)}
      />
    );
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
  return (
    <ProjectOverview
      key={project}
      name={project}
      onArchived={onArchived}
      onCloned={onCloned}
      onDelete={() => onDelete(project, false)}
      onRename={() => onRename(project)}
    />
  );
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
  const dismiss = useCallback(() => setFailure(null), []);
  return { busy, failure, act, dismiss };
}

/** An archived project: read-only, with Restore (ADR-0015). */
function ArchivedOverview({
  name,
  path,
  onRestored,
  onDelete,
}: {
  name: string;
  path: string;
  onRestored: (moved: ProjectMoved) => void;
  onDelete: () => void;
}) {
  const restore = useProjectAction(() => engine.project.restore(name), onRestored);
  return (
    <section className="panel">
      <div className="panel-head">
        <h2>{name}</h2>
        <div className="row">
          <button type="button" onClick={() => void restore.act()} disabled={restore.busy}>
            {restore.busy ? "Restoring…" : "Restore"}
          </button>
          <button type="button" className="quiet danger" onClick={onDelete}>
            Delete…
          </button>
        </div>
      </div>
      <p className="muted">
        Archived. Restore it to chat in it, search it or change its permissions again.
      </p>
      <dl className="facts">
        <dt>Folder</dt>
        <dd className="path">{path}</dd>
      </dl>
      {restore.failure && (
        <Failure message={restore.failure.message} kind={restore.failure.kind} onDismiss={restore.dismiss} />
      )}
    </section>
  );
}

function ProjectOverview({
  name,
  onArchived,
  onCloned,
  onDelete,
  onRename,
}: {
  name: string;
  onArchived: (moved: ProjectMoved) => void;
  onCloned: (clone: ProjectDetails) => void;
  onDelete: () => void;
  onRename: () => void;
}) {
  const archive = useProjectAction(() => engine.project.archive(name), onArchived);
  const clone = useProjectAction(() => engine.project.clone(name), onCloned);
  const [details, reload] = useLoad(() => engine.project.show(name), name);

  if (details.state === "loading") return <p className="muted panel">Loading {name}…</p>;
  // Archive and Delete do not read the project, so they are offered even when its details
  // cannot load.
  const archiveButton = (
    <>
      <button
        type="button"
        className="quiet"
        onClick={() => void archive.act()}
        disabled={archive.busy}
        title="Move it to Projects/Archive/; it can be restored"
      >
        {archive.busy ? "Archiving…" : "Archive"}
      </button>
      <button type="button" className="quiet danger" onClick={onDelete} title="Move it to the Trash">
        Delete…
      </button>
    </>
  );
  if (details.state === "failed")
    return (
      <section className="panel">
        <div className="panel-head">
          <h2>{name}</h2>
          <div className="row">{archiveButton}</div>
        </div>
        <Failure message={details.message} kind={details.kind} />
        {archive.failure && (
        <Failure message={archive.failure.message} kind={archive.failure.kind} onDismiss={archive.dismiss} />
      )}
      </section>
    );

  const p = details.value;
  return (
    <section className="panel">
      <div className="panel-head">
        <h2>{p.display_name}</h2>
        <div className="row">
          <button type="button" className="quiet" onClick={onRename} title="Change its name and folder">
            Rename…
          </button>
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
      {archive.failure && (
        <Failure message={archive.failure.message} kind={archive.failure.kind} onDismiss={archive.dismiss} />
      )}
      {clone.failure && (
        <Failure message={clone.failure.message} kind={clone.failure.kind} onDismiss={clone.dismiss} />
      )}
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
  const [whereModels, setWhereModels] = useState(false);
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
        {installed.length === 0 && (
          <button
            type="button"
            className="help-button"
            aria-label="Where are my models?"
            aria-expanded={whereModels}
            title="Where are my models?"
            onClick={() => setWhereModels((open) => !open)}
          >
            ?
          </button>
        )}
      </p>
      {installed.length === 0 && whereModels && <WhereModels />}
      {failure && <Failure message={failure.message} kind={failure.kind} />}
    </form>
  );
}

/**
 * Why the list of models can be empty: each Ollama keeps its own models, and only the one
 * running now is asked.
 */
function WhereModels() {
  return (
    <div className="help-text" role="note" aria-label="Where your models are">
      <p>
        Each Ollama keeps its own models, and this list shows only those of the Ollama running
        now. Nothing runs until a chat session starts one, unless Ollama runs as a service.
      </p>
      <ul>
        <li>
          Ollama&apos;s installer runs it as a <strong>system service</strong>, with models in{" "}
          <code>/usr/share/ollama/.ollama/models</code>.
        </li>
        <li>
          When nothing is running, AI Workspace <strong>starts Ollama as you</strong>, with
          models in <code>~/.ollama/models</code>.
        </li>
        <li>
          <strong>Ollama in Podman</strong> keeps its own, in the{" "}
          <code>ai-workspace-ollama</code> volume.
        </li>
      </ul>
      <p>
        To use the models you downloaded through the service, start it (
        <code>sudo systemctl start ollama</code>) and start the chat session again. Or download a
        model into the Ollama that runs: <code>ollama pull &lt;model&gt;</code>.
      </p>
    </div>
  );
}
