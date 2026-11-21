import { useState, type FormEvent } from "react";

import { engine, type RootSaved, type SwitchDone, type SwitchOffer } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { useLoad } from "../lib/useLoad";
import { Failure } from "./Failure";
import { InitButton } from "./InitButton";

/** How the workspace changed: `folder` when another folder may now be in use. */
export type WorkspaceChange = { folder: boolean; sessionClosed: boolean };

type Props = {
  /** The workspace changed (initialized, another folder chosen, or switched): read it again. */
  onChanged: (change: WorkspaceChange) => void;
};

/**
 * The workspace configuration: what `Config/workspace.toml` says and which folder is in use;
 * choosing another workspace folder; initializing a workspace that has no configuration.
 */
export function ConfigPanel({ onChanged }: Props) {
  const [settings, reload] = useLoad(() => engine.workspace.config(), "config");
  // Saving a folder changes what Switch now would use: read the offer again.
  const [offerKey, setOfferKey] = useState(0);
  const changed = (change: WorkspaceChange) => {
    reload();
    onChanged(change);
  };

  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Configuration</h2>
        <button type="button" className="quiet" onClick={reload} title="Read the file again">
          Reload
        </button>
      </header>

      <SwitchNow key={offerKey} onSwitched={changed} />

      {settings.state === "loading" && <p className="muted">Reading the configuration…</p>}
      {settings.state === "failed" && (
        <>
          <Failure message={settings.message} kind={settings.kind} />
          {settings.kind === "ConfigMissing" && (
            <>
              <p className="muted">
                The workspace has no configuration file yet. Initializing it creates the
                standard folders and a configuration with the defaults.
              </p>
              <div>
                <InitButton onDone={() => changed({ folder: false, sessionClosed: false })} />
              </div>
            </>
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
            <dt>Runs Ollama with</dt>
            <dd>{settings.value.runtime}</dd>
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
      <WorkspaceFolder
        onSaved={(change) => {
          setOfferKey((key) => key + 1);
          changed(change);
        }}
      />
    </section>
  );
}

/**
 * While AI_WORKSPACE_HOME decides the folder, Switch now is offered straight away: it uses the
 * saved folder (else ~/AI) until the app restarts.
 */
function SwitchNow({ onSwitched }: { onSwitched: (change: WorkspaceChange) => void }) {
  const [offer, reload] = useLoad(() => engine.workspace.switchOffer(), "switch offer");
  const [busy, setBusy] = useState(false);
  const [switched, setSwitched] = useState<SwitchDone | null>(null);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  async function switchNow() {
    setBusy(true);
    setFailure(null);
    try {
      const result = await engine.workspace.switchToSaved();
      setSwitched(result);
      reload();
      onSwitched({ folder: true, sessionClosed: result.session_closed !== null });
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  if (switched) {
    return (
      <div className="notice" role="status" aria-label="Switched">
        <p>Now using {switched.root}.</p>
        <p className="small">Chosen by {switched.root_source}.</p>
        {switched.session_closed && (
          <p>The chat session in {switched.session_closed.project} was ended.</p>
        )}
      </div>
    );
  }
  if (offer.state !== "ready" || offer.value === null) return null;
  const choice: SwitchOffer = offer.value;
  return (
    <div className="notice" role="status" aria-label="Variable decides">
      <p>
        AI_WORKSPACE_HOME is set to {choice.variable_root} and decides the folder. Your saved
        folder is {choice.saved_root} ({choice.saved_source}).
      </p>
      <p className="small">
        Switch now uses the saved folder until the app restarts; an open chat is ended first. To
        use it every time, start the app without the variable.
      </p>
      <div>
        <button type="button" onClick={switchNow} disabled={busy}>
          {busy ? "Switching…" : "Switch now"}
        </button>
      </div>
      {failure && <Failure message={failure.message} kind={failure.kind} />}
    </div>
  );
}

/** Choose another workspace folder; it is saved in the per-user settings file. */
function WorkspaceFolder({ onSaved }: { onSaved: (change: WorkspaceChange) => void }) {
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState<RootSaved | null>(null);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  async function save(event: FormEvent) {
    event.preventDefault();
    if (path.trim() === "" || busy) return;
    setBusy(true);
    setFailure(null);
    setSaved(null);
    try {
      const result = await engine.workspace.setRoot(path);
      setSaved(result);
      setPath("");
      onSaved({ folder: true, sessionClosed: result.session_closed !== null });
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <h3>Workspace folder</h3>
      <p className="muted small">
        Use another existing folder as the workspace. It is remembered in
        ~/.config/ai-workspace/config.toml; an open chat session is ended first.
      </p>
      <form className="search" onSubmit={save} aria-label="Workspace folder">
        <input
          value={path}
          onChange={(event) => setPath(event.target.value)}
          placeholder="Folder, e.g. ~/ai-local/workspaces"
          aria-label="New workspace folder"
        />
        <button type="submit" disabled={busy || path.trim() === ""}>
          {busy ? "Saving…" : "Use this folder"}
        </button>
      </form>
      {failure && <Failure message={failure.message} kind={failure.kind} />}
      {saved && (
        <div className="notice" role="status">
          <p>Workspace folder set to {saved.root}.</p>
          {saved.session_closed && (
            <p>The chat session in {saved.session_closed.project} was ended.</p>
          )}
          {saved.env_override && (
            <p>
              AI_WORKSPACE_HOME is set to {saved.env_override} and still decides the folder: use
              Switch now above to use the saved folder until the app restarts, or start the app
              without the variable to use it every time.
            </p>
          )}
        </div>
      )}
    </>
  );
}
