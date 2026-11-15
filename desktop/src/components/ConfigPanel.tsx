import { engine } from "../lib/engine";
import { useLoad } from "../lib/useLoad";
import { Failure } from "./Failure";

/**
 * The workspace configuration, read-only: what `Config/workspace.toml` says, and which folder
 * is in use. Settings are changed in the file itself for now.
 */
export function ConfigPanel() {
  const [settings, reload] = useLoad(() => engine.workspace.config(), "config");

  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Configuration</h2>
        <button type="button" className="quiet" onClick={reload} title="Read the file again">
          Reload
        </button>
      </header>

      {settings.state === "loading" && <p className="muted">Reading the configuration…</p>}
      {settings.state === "failed" && (
        <>
          <Failure message={settings.message} kind={settings.kind} />
          {settings.kind === "ConfigMissing" && (
            <p className="muted">
              The workspace has no configuration file yet. Initializing the workspace creates
              one with the defaults.
            </p>
          )}
        </>
      )}
      {settings.state === "ready" && (
        <>
          <dl className="facts">
            <dt>Workspace folder</dt>
            <dd className="path">{settings.value.active_root}</dd>
            <dt>Chosen by</dt>
            <dd>{settings.value.root_source}</dd>
            <dt>Configuration file</dt>
            <dd className="path">{settings.value.config_file}</dd>
            <dt>Default model</dt>
            <dd>{settings.value.default_model}</dd>
            <dt>Project file</dt>
            <dd>{settings.value.project_metadata_file}</dd>
            <dt>New project folders</dt>
            <dd>{settings.value.default_project_directories.join(", ")}</dd>
          </dl>

          <h3>Recorded in the file</h3>
          <p className="muted small">
            The folders as written when the file was created. They are for reference: the
            folders in use always follow the workspace folder above.
          </p>
          <dl className="facts">
            <dt>Root</dt>
            <dd className="path">{settings.value.recorded_root}</dd>
            <dt>Base</dt>
            <dd className="path">{settings.value.recorded_base}</dd>
            <dt>Projects</dt>
            <dd className="path">{settings.value.recorded_projects}</dd>
            <dt>Cache</dt>
            <dd className="path">{settings.value.recorded_cache}</dd>
            <dt>Logs</dt>
            <dd className="path">{settings.value.recorded_logs}</dd>
            <dt>Config</dt>
            <dd className="path">{settings.value.recorded_config}</dd>
          </dl>
        </>
      )}
    </section>
  );
}
