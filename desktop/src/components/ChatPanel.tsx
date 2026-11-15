import { useEffect, useRef, useState, type FormEvent } from "react";

import { engine, messageOf, type SessionInfo } from "../lib/engine";
import { counted, fileName } from "../lib/words";

type Line = { who: "you" | "ai" | "note"; text: string };

type Props = {
  project: string | null;
  session: SessionInfo | null;
  onSession: (session: SessionInfo | null) => void;
};

/**
 * A chat in one project, driving the core's session: opening it grants the project's default
 * permissions and builds the first context; every message rebuilds the context from what was
 * asked; closing revokes temporary permissions.
 */
export function ChatPanel({ project, session, onSession }: Props) {
  const [lines, setLines] = useState<Line[]>([]);
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState<null | "opening" | "waiting" | "closing">(null);
  const [error, setError] = useState<string | null>(null);
  const end = useRef<HTMLDivElement>(null);

  useEffect(() => {
    end.current?.scrollIntoView?.({ block: "end" });
  }, [lines, busy]);

  async function open() {
    if (project === null) return;
    setBusy("opening");
    setError(null);
    try {
      const opened = await engine.chat.open(project);
      onSession(opened);
      setLines([]);
    } catch (thrown) {
      setError(messageOf(thrown));
    } finally {
      setBusy(null);
    }
  }

  async function close() {
    setBusy("closing");
    setError(null);
    try {
      const closed = await engine.chat.close();
      onSession(null);
      setLines(
        closed
          ? [
              {
                who: "note",
                text: `Session in ${closed.project} closed; ${counted(closed.revoked, "temporary permission")} revoked.`,
              },
            ]
          : [],
      );
    } catch (thrown) {
      setError(messageOf(thrown));
    } finally {
      setBusy(null);
    }
  }

  async function send(event: FormEvent) {
    event.preventDefault();
    const message = draft.trim();
    if (message === "" || busy !== null) return;
    setDraft("");
    setError(null);
    setLines((all) => [...all, { who: "you", text: message }]);
    setBusy("waiting");
    try {
      const reply = await engine.chat.send(message);
      setLines((all) => [...all, { who: "ai", text: reply }]);
    } catch (thrown) {
      setLines((all) => [
        ...all,
        { who: "note", text: `No answer: ${messageOf(thrown)}. You can try again.` },
      ]);
    } finally {
      setBusy(null);
    }
  }

  if (session === null) {
    return (
      <section className="panel">
        <h2>Chat</h2>
        {lines.map((line, index) => (
          <p key={index} className="muted">
            {line.text}
          </p>
        ))}
        {project === null ? (
          <p className="muted">Choose a project on the left to chat in it.</p>
        ) : (
          <>
            <p>
              Start a session in <strong>{project}</strong>. The project&apos;s files become the
              model&apos;s context; nothing leaves this computer.
            </p>
            <button type="button" onClick={open} disabled={busy !== null}>
              {busy === "opening" ? "Starting… (this can start Ollama)" : `Start session in ${project}`}
            </button>
          </>
        )}
        {error && (
          <p className="failure" role="alert">
            {error}
          </p>
        )}
      </section>
    );
  }

  const files = session.base_files.length + session.project_files.length + session.session_files.length;
  return (
    <section className="panel chat">
      <header className="panel-head">
        <h2>Chat · {session.project}</h2>
        <button type="button" className="quiet" onClick={close} disabled={busy !== null}>
          {busy === "closing" ? "Closing…" : "End session"}
        </button>
      </header>

      <p className={session.agent_included ? "agent ok" : "agent"} title={session.agent}>
        Model <strong>{session.model}</strong> · {session.agent}
      </p>

      <details className="context">
        <summary>
          {session.runtime} · {counted(files, "context file")}
        </summary>
        <ContextList title="Base" files={session.base_files} />
        <ContextList title="Project" files={session.project_files} />
        <ContextList title="Session" files={session.session_files} />
        <ContextList title="Skipped (names look like secrets)" files={session.skipped_files} />
        <p className="muted small">Budget: {session.budget}</p>
        {session.truncated_files.length > 0 && (
          <div className="context-group">
            <div className="label">Cut to fit</div>
            <ul>
              {session.truncated_files.map((file) => (
                <li key={file}>{file}</li>
              ))}
            </ul>
          </div>
        )}
      </details>

      {project !== null && project !== session.project && (
        <p className="muted small">
          This session is in {session.project}. End it to start one in {project}.
        </p>
      )}

      <div className="transcript" aria-live="polite">
        {lines.length === 0 && session.can_chat && (
          <p className="muted">Ask a question about {session.project}.</p>
        )}
        {lines.map((line, index) => (
          <div key={index} className={`line ${line.who}`}>
            {line.text}
          </div>
        ))}
        {busy === "waiting" && <div className="line ai thinking">Thinking…</div>}
        <div ref={end} />
      </div>

      {session.can_chat ? (
        <form className="composer" onSubmit={send}>
          <textarea
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter" && !event.shiftKey) {
                event.preventDefault();
                event.currentTarget.form?.requestSubmit();
              }
            }}
            placeholder="Message (Enter to send, Shift+Enter for a new line)"
            aria-label="Message"
            rows={2}
          />
          <button type="submit" disabled={draft.trim() === "" || busy !== null}>
            Send
          </button>
        </form>
      ) : (
        <p className="failure">No local runtime is running, so the model cannot answer. Install or start Ollama, then start the session again.</p>
      )}
      {error && (
        <p className="failure" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}

function ContextList({ title, files }: { title: string; files: string[] }) {
  if (files.length === 0) return null;
  return (
    <div className="context-group">
      <div className="label">{title}</div>
      <ul>
        {files.map((file) => (
          <li key={file} title={file}>
            {fileName(file)}
          </li>
        ))}
      </ul>
    </div>
  );
}
