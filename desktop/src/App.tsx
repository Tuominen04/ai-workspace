import { useState } from "react";

import { ChatPanel } from "./components/ChatPanel";
import { ConfigPanel } from "./components/ConfigPanel";
import { DeleteDialog } from "./components/DeleteDialog";
import { RenameDialog } from "./components/RenameDialog";
import { PermissionsPanel } from "./components/PermissionsPanel";
import { Failure } from "./components/Failure";
import { Overview } from "./components/Overview";
import { SearchPanel } from "./components/SearchPanel";
import { Sidebar } from "./components/Sidebar";
import { ValidationPanel } from "./components/ValidationPanel";
import {
  engine,
  isDesktopRuntime,
  type ProjectDeleted,
  type ProjectDetails,
  type ProjectMoved,
  type ProjectRenamed,
  type SessionInfo,
} from "./lib/engine";
import { useLoad } from "./lib/useLoad";

// The whole workspace and a project each have their own tabs; a project opens on its chat.
const WORKSPACE_TABS = ["Overview", "Validation", "Search", "Configuration"] as const;
const PROJECT_TABS = ["Chat", "Overview", "Search", "Permissions"] as const;
// An archived project is only looked at, restored or deleted (ADR-0015).
const ARCHIVED_TABS = ["Overview"] as const;
type Tab = (typeof WORKSPACE_TABS)[number] | (typeof PROJECT_TABS)[number];

/** The tab a selection opens on: a project's chat, or the whole workspace's overview. */
function firstTab(project: string | null): Tab {
  return project === null ? "Overview" : "Chat";
}

export default function App() {
  if (!isDesktopRuntime()) {
    return (
      <main className="no-runtime">
        <h1>AI Workspace</h1>
        <p>
          This is the desktop app&apos;s interface on its own. Start it with{" "}
          <code>npm run tauri dev</code> to reach your workspace.
        </p>
      </main>
    );
  }
  return <Workspace />;
}

function Workspace() {
  const [status, refresh] = useLoad(() => engine.workspace.status(), "status");
  const [project, setProject] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("Overview");
  // Kept here, not in the chat panel, so switching tabs or projects does not lose the open session.
  const [session, setSession] = useState<SessionInfo | null>(null);
  const [archived, setArchived] = useState<string | null>(null);
  // The sidebar lists either the projects or the archived projects.
  const [showArchived, setShowArchived] = useState(false);
  // The project being deleted, while its confirmation is open.
  const [deleting, setDeleting] = useState<{ name: string; archived: boolean } | null>(null);
  // The project being renamed, while its dialog is open.
  const [renaming, setRenaming] = useState<string | null>(null);
  // What the last delete did, shown until the next selection.
  const [notice, setNotice] = useState<string | null>(null);
  const tabs: readonly Tab[] =
    archived !== null ? ARCHIVED_TABS : project === null ? WORKSPACE_TABS : PROJECT_TABS;

  function select(next: string | null) {
    setNotice(null);
    setArchived(null);
    setShowArchived(false);
    setProject(next);
    setTab(firstTab(next));
  }

  /** A project was deleted: the whole workspace is shown, with what happened. */
  function deletedDone(deleted: ProjectDeleted) {
    setDeleting(null);
    if (deleted.session_closed) setSession(null);
    refresh();
    select(null);
    setNotice(
      deleted.erased
        ? `Erased ${deleted.name}.`
        : `Moved ${deleted.name} to the Trash. Restore it from the Files app's Trash if you need it back.`,
    );
  }

  /** A project was renamed: it is selected under its new name, on its Overview. */
  function renamedDone(renamed: ProjectRenamed) {
    setRenaming(null);
    if (renamed.session_closed) setSession(null);
    refresh();
    select(renamed.project.name);
    setTab("Overview");
  }

  function selectArchived(name: string) {
    setNotice(null);
    setProject(null);
    setArchived(name);
    setTab("Overview");
  }

  /** A project was archived: the whole workspace is shown again, as the project is gone. */
  function archivedDone(moved: ProjectMoved) {
    if (moved.session_closed) setSession(null);
    refresh();
    select(null);
  }

  /** A project was cloned: the clone is listed and opened on its chat. */
  function clonedDone(clone: ProjectDetails) {
    refresh();
    select(clone.name);
  }

  /** A project was restored: it is a normal project again, opened on its chat. */
  function restoredDone(moved: ProjectMoved) {
    refresh();
    select(moved.name);
  }

  return (
    <div className="app">
      <Sidebar
        status={status}
        selected={project}
        sessionProject={session?.project ?? null}
        selectedArchived={archived}
        showArchived={showArchived}
        onShowArchived={setShowArchived}
        onSelect={select}
        onSelectArchived={selectArchived}
        onArchived={archivedDone}
        onRestored={restoredDone}
        onCloned={clonedDone}
        onDelete={(name, isArchived) => setDeleting({ name, archived: isArchived })}
        onRename={setRenaming}
        onRefresh={refresh}
        onCreated={(created) => {
          refresh();
          select(created);
        }}
      />
      <main className="main">
        <div className="tabs" role="tablist">
          {tabs.map((name) => (
            <button
              type="button"
              role="tab"
              key={name}
              aria-selected={tab === name}
              className={tab === name ? "tab active" : "tab"}
              onClick={() => setTab(name)}
            >
              {name}
              {name === "Chat" && session !== null && <span className="dot" aria-label="session open" />}
            </button>
          ))}
        </div>
        <div className="content">
          {notice && (
            <p className="notice" role="status">
              {notice}
            </p>
          )}
          {tab === "Overview" &&
            (status.state === "ready" ? (
              <Overview
                status={status.value}
                project={project}
                archived={archived}
                onArchived={archivedDone}
                onRestored={restoredDone}
                onCloned={clonedDone}
                onDelete={(name, isArchived) => setDeleting({ name, archived: isArchived })}
                onRename={setRenaming}
              />
            ) : status.state === "failed" ? (
              <section className="panel">
                <Failure message={status.message} kind={status.kind} />
              </section>
            ) : (
              <p className="muted panel">Reading the workspace…</p>
            ))}
          {tab === "Validation" && <ValidationPanel onChanged={refresh} />}
          {tab === "Search" && <SearchPanel project={project} />}
          {/* Always mounted, only hidden on other tabs: the conversation, a half-typed message
              and a reply still on its way survive switching tabs, until the session ends or the
              app closes. */}
          <div className="tab-page" hidden={project === null || tab !== "Chat"}>
            <ChatPanel project={project} session={session} onSession={setSession} />
          </div>
          {tab === "Permissions" && project !== null && <PermissionsPanel project={project} />}
          {tab === "Configuration" && (
            <ConfigPanel
              onChanged={(change) => {
                if (change.sessionClosed) setSession(null);
                // Configuration is a whole-workspace tab, so no project is selected here; stay on
                // it so its confirmation stays visible.
                if (change.folder) setProject(null);
                refresh();
              }}
            />
          )}
        </div>
      </main>
      {renaming && (
        <RenameDialog name={renaming} onRenamed={renamedDone} onCancel={() => setRenaming(null)} />
      )}
      {deleting && (
        <DeleteDialog
          name={deleting.name}
          archived={deleting.archived}
          onDeleted={deletedDone}
          onCancel={() => setDeleting(null)}
        />
      )}
    </div>
  );
}
