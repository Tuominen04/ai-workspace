import { useState } from "react";

import type { WorkspaceStatus } from "../lib/engine";
import { applyTheme, isTheme, savedTheme, THEMES, themeLabel, type Theme } from "../lib/theme";
import type { Loaded } from "../lib/useLoad";
import { Failure } from "./Failure";

type Props = {
  status: Loaded<WorkspaceStatus>;
  selected: string | null;
  onSelect: (project: string | null) => void;
  onRefresh: () => void;
};

/** The workspace folder and its projects. Choosing one scopes the other panels to it. */
export function Sidebar({ status, selected, onSelect, onRefresh }: Props) {
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
            {status.value.projects.length === 0 && (
              <p className="muted small">
                No projects yet. Creating projects from this window is coming; for now a project is
                made with the command line from source.
              </p>
            )}
          </nav>
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
