import { useState, type FormEvent } from "react";

import { engine, type WorkspaceStatus } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { applyTheme, isTheme, savedTheme, THEMES, themeLabel, type Theme } from "../lib/theme";
import type { Loaded } from "../lib/useLoad";
import { Failure } from "./Failure";

type Props = {
  status: Loaded<WorkspaceStatus>;
  selected: string | null;
  onSelect: (project: string | null) => void;
  onRefresh: () => void;
  /** A project was created: read the workspace again and select it. */
  onCreated: (project: string) => void;
};

/** The workspace folder and its projects. Choosing one scopes the other panels to it. */
export function Sidebar({ status, selected, onSelect, onRefresh, onCreated }: Props) {
  return (
    <aside className="sidebar">
      <header>
        <h1>AI Workspace</h1>
        <button type="button" className="quiet" onClick={onRefresh} title="Read the workspace again">
          Refresh
        </button>
      </header>

      {status.state === "loading" && <p className="muted">Reading the workspace…</p>}
      {status.state === "failed" && <Failure message={status.message} kind={status.kind} />}
      {status.state === "ready" && (
        <>
          <section className="root" aria-label="Workspace folder">
            <div className="label">Workspace</div>
            <div className="path" title={status.value.root}>
              {status.value.root}
            </div>
            <div className="muted small">Chosen by {status.value.root_source}</div>
          </section>

          <nav aria-label="Projects">
            <div className="label">Projects</div>
            <button
              type="button"
              className={selected === null ? "item active" : "item"}
              onClick={() => onSelect(null)}
            >
              Whole workspace
            </button>
            {status.value.projects.map((project) => (
              <button
                type="button"
                key={project.name}
                className={selected === project.name ? "item active" : "item"}
                onClick={() => onSelect(project.name)}
                title={project.path}
              >
                {project.name}
              </button>
            ))}
            {status.value.projects.length === 0 && <p className="muted small">No projects yet.</p>}
          </nav>
          <NewProject onCreated={onCreated} />
        </>
      )}

      <ThemePicker />
    </aside>
  );
}

/** System, Light or Dark. Takes effect at once and is remembered on this computer. */
function ThemePicker() {
  const [theme, setTheme] = useState<Theme>(savedTheme);
  return (
    <label className="theme muted small">
      Theme
      <select
        value={theme}
        onChange={(event) => {
          const next = event.target.value;
          if (!isTheme(next)) return;
          applyTheme(next);
          setTheme(next);
        }}
      >
        {THEMES.map((option) => (
          <option key={option} value={option}>
            {themeLabel(option)}
          </option>
        ))}
      </select>
    </label>
  );
}

/** "New project": a name and an optional description, created in the workspace. */
function NewProject({ onCreated }: { onCreated: (project: string) => void }) {
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  if (!open) {
    return (
      <button type="button" className="quiet new-project" onClick={() => setOpen(true)}>
        + New project
      </button>
    );
  }

  async function create(event: FormEvent) {
    event.preventDefault();
    if (name.trim() === "") return;
    setBusy(true);
    setFailure(null);
    try {
      const created = await engine.project.create(name, description);
      setOpen(false);
      setName("");
      setDescription("");
      onCreated(created.name);
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  return (
    <form className="new-project-form" onSubmit={create} aria-label="New project">
      <div className="label">New project</div>
      <input
        value={name}
        onChange={(event) => setName(event.target.value)}
        placeholder="Name"
        aria-label="Project name"
        autoFocus
      />
      <input
        value={description}
        onChange={(event) => setDescription(event.target.value)}
        placeholder="Description (optional)"
        aria-label="Project description"
      />
      <div className="row">
        <button type="submit" disabled={busy || name.trim() === ""}>
          {busy ? "Creating…" : "Create"}
        </button>
        <button type="button" className="quiet" onClick={() => setOpen(false)} disabled={busy}>
          Cancel
        </button>
      </div>
      {failure && <Failure message={failure.message} kind={failure.kind} />}
    </form>
  );
}
