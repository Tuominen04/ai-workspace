import { useState } from "react";

import { ChatPanel } from "./components/ChatPanel";
import { ConfigPanel } from "./components/ConfigPanel";
import { PermissionsPanel } from "./components/PermissionsPanel";
import { Failure } from "./components/Failure";
import { Overview } from "./components/Overview";
import { SearchPanel } from "./components/SearchPanel";
import { Sidebar } from "./components/Sidebar";
import { ValidationPanel } from "./components/ValidationPanel";
import { engine, isDesktopRuntime, type SessionInfo } from "./lib/engine";
import { useLoad } from "./lib/useLoad";

const TABS = ["Overview", "Validation", "Search", "Chat", "Permissions", "Configuration"] as const;
type Tab = (typeof TABS)[number];

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
  // Kept here, not in the chat panel, so switching tabs does not lose the open session.
  const [session, setSession] = useState<SessionInfo | null>(null);

  return (
    <div className="app">
      <Sidebar
        status={status}
        selected={project}
        onSelect={setProject}
        onRefresh={refresh}
        onCreated={(created) => {
          refresh();
          setProject(created);
          setTab("Overview");
        }}
      />
      <main className="main">
        <div className="tabs" role="tablist">
          {TABS.map((name) => (
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
          {tab === "Overview" &&
            (status.state === "ready" ? (
              <Overview status={status.value} project={project} />
            ) : status.state === "failed" ? (
              <section className="panel">
                <Failure message={status.message} kind={status.kind} />
              </section>
            ) : (
              <p className="muted panel">Reading the workspace…</p>
            ))}
          {tab === "Validation" && <ValidationPanel onChanged={refresh} />}
          {tab === "Search" && <SearchPanel project={project} />}
          {tab === "Chat" && <ChatPanel project={project} session={session} onSession={setSession} />}
          {tab === "Permissions" && <PermissionsPanel project={project} />}
          {tab === "Configuration" && (
            <ConfigPanel
              onChanged={(saved) => {
                if (saved?.session_closed) setSession(null);
                // Another workspace has other projects; the old selection may not exist there.
                if (saved) setProject(null);
                refresh();
              }}
            />
          )}
        </div>
      </main>
    </div>
  );
}
