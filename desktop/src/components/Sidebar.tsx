import { useEffect, useRef, useState, type FormEvent, type MouseEvent } from "react";

import { engine, type ProjectDetails, type ProjectMoved, type WorkspaceStatus } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { applyTheme, isTheme, savedTheme, THEMES, themeLabel, type Theme } from "../lib/theme";
import type { Loaded } from "../lib/useLoad";
import { Failure } from "./Failure";

type Props = {
  status: Loaded<WorkspaceStatus>;
  selected: string | null;
  /** The project with an open chat session, marked with a dot. */
  sessionProject: string | null;
  /** The selected archived project, if one is. */
  selectedArchived: string | null;
  /** Whether the sidebar shows the archived projects instead of the projects. */
  showArchived: boolean;
  onShowArchived: (show: boolean) => void;
  onSelect: (project: string | null) => void;
  onSelectArchived: (project: string) => void;
  onRefresh: () => void;
  /** A project was created: read the workspace again and select it. */
  onCreated: (project: string) => void;
  onArchived: (moved: ProjectMoved) => void;
  onRestored: (moved: ProjectMoved) => void;
  onCloned: (clone: ProjectDetails) => void;
  /** Asks to delete a project (or an archived one); the confirmation is the app's. */
  onDelete: (name: string, archived: boolean) => void;
};

/** A right-click menu open on one project, at the pointer. */
type Menu = { x: number; y: number; name: string; archived: boolean };

/** The whole workspace and its projects, or the archived projects. Choosing one scopes the
 * panels to it; the workspace folder itself is shown in the whole workspace's Overview. A
 * project's right-click menu archives it; an archived project's restores it (ADR-0015). */
export function Sidebar({
  status,
  selected,
  sessionProject,
  selectedArchived,
  showArchived,
  onShowArchived,
  onSelect,
  onSelectArchived,
  onRefresh,
  onCreated,
  onArchived,
  onRestored,
  onCloned,
  onDelete,
}: Props) {
  const [menu, setMenu] = useState<Menu | null>(null);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  function openMenu(event: MouseEvent, name: string, archived: boolean) {
    event.preventDefault();
    setFailure(null);
    // From the keyboard (context-menu key, Shift+F10) there is no pointer: use the item.
    const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const fromPointer = event.clientX !== 0 || event.clientY !== 0;
    setMenu({
      x: fromPointer ? event.clientX : box.left + 16,
      y: fromPointer ? event.clientY : box.bottom,
      name,
      archived,
    });
  }

  async function run<T>(action: () => Promise<T>, done: (result: T) => void) {
    setMenu(null);
    try {
      done(await action());
    } catch (thrown) {
      setFailure(failureOf(thrown));
    }
  }

  const archivedCount = status.state === "ready" ? status.value.archived.length : 0;

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
      {status.state === "ready" && !showArchived && (
        <>
          <nav aria-label="Projects">
            <button
              type="button"
              className={selected === null && selectedArchived === null ? "item active" : "item"}
              onClick={() => onSelect(null)}
              title={status.value.root}
            >
              Whole workspace
            </button>
            <div className="label projects-label">Projects</div>
            {status.value.projects.map((project) => (
              <button
                type="button"
                key={project.name}
                className={selected === project.name ? "item active" : "item"}
                onClick={() => onSelect(project.name)}
                onContextMenu={(event) => openMenu(event, project.name, false)}
                title={project.path}
              >
                {project.name}
                {project.name === sessionProject && <span className="dot" aria-label="session open" />}
              </button>
            ))}
            {status.value.projects.length === 0 && <p className="muted small">No projects yet.</p>}
          </nav>
          <NewProject onCreated={onCreated} />
        </>
      )}
      {status.state === "ready" && showArchived && (
        <nav aria-label="Archived projects">
          <button type="button" className="quiet back" onClick={() => onShowArchived(false)}>
            ← Projects
          </button>
          <div className="label projects-label">Archived</div>
          {status.value.archived.map((project) => (
            <button
              type="button"
              key={project.name}
              className={selectedArchived === project.name ? "item active" : "item"}
              onClick={() => onSelectArchived(project.name)}
              onContextMenu={(event) => openMenu(event, project.name, true)}
              title={project.path}
            >
              {project.name}
            </button>
          ))}
          {status.value.archived.length === 0 && <p className="muted small">No archived projects.</p>}
        </nav>
      )}

      {failure && <Failure message={failure.message} kind={failure.kind} />}
      {menu && (
        <ProjectMenu
          menu={menu}
          onClose={() => setMenu(null)}
          onArchive={() => void run(() => engine.project.archive(menu.name), onArchived)}
          onClone={() => void run(() => engine.project.clone(menu.name), onCloned)}
          onDelete={() => {
            setMenu(null);
            onDelete(menu.name, menu.archived);
          }}
          onRestore={() => void run(() => engine.project.restore(menu.name), onRestored)}
        />
      )}

      <div className="sidebar-foot">
        {status.state === "ready" && !showArchived && archivedCount > 0 && (
          <button type="button" className="item archived-toggle" onClick={() => onShowArchived(true)}>
            Archived ({archivedCount})
          </button>
        )}
        <ThemePicker />
      </div>
    </aside>
  );
}

/** The right-click menu: Clone, Archive and Delete for a project; Restore and Delete for an
 * archived one. Closes on a
 * choice, a click elsewhere, Escape or scrolling. */
function ProjectMenu({
  menu,
  onClose,
  onArchive,
  onClone,
  onRestore,
  onDelete,
}: {
  menu: Menu;
  onClose: () => void;
  onArchive: () => void;
  onClone: () => void;
  onRestore: () => void;
  onDelete: () => void;
}) {
  const first = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    first.current?.focus();
    const close = (event: Event) => {
      if (event instanceof KeyboardEvent && event.key !== "Escape") return;
      if (event.type === "mousedown" && (event.target as Element).closest?.(".context-menu")) return;
      onClose();
    };
    document.addEventListener("mousedown", close);
    document.addEventListener("keydown", close);
    window.addEventListener("blur", close);
    window.addEventListener("resize", close);
    document.addEventListener("scroll", close, true);
    return () => {
      document.removeEventListener("mousedown", close);
      document.removeEventListener("keydown", close);
      window.removeEventListener("blur", close);
      window.removeEventListener("resize", close);
      document.removeEventListener("scroll", close, true);
    };
  }, [onClose]);

  return (
    <div
      className="context-menu"
      role="menu"
      aria-label={`Actions for ${menu.name}`}
      style={{ left: menu.x, top: menu.y }}
    >
      {menu.archived ? (
        <button type="button" role="menuitem" ref={first} onClick={onRestore}>
          Restore
        </button>
      ) : (
        <>
          <button type="button" role="menuitem" ref={first} onClick={onClone}>
            Clone
          </button>
          <button type="button" role="menuitem" onClick={onArchive}>
            Archive
          </button>
        </>
      )}
      <button type="button" role="menuitem" className="danger-item" onClick={onDelete}>
        Delete…
      </button>
    </div>
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
