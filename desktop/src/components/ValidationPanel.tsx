import { engine } from "../lib/engine";
import { useLoad } from "../lib/useLoad";
import { counted, fixText, issueWhere } from "../lib/words";
import { Failure } from "./Failure";

/** A validation run: pass, or every issue with how to clear it. Never blocks anything itself. */
export function ValidationPanel() {
  const [validation, rerun] = useLoad(() => engine.workspace.validate(), "validate");

  return (
    <section className="panel">
      <header className="panel-head">
        <h2>Validation</h2>
        <button type="button" onClick={rerun}>
          Validate again
        </button>
      </header>

      {validation.state === "loading" && <p className="muted">Checking the workspace…</p>}
      {validation.state === "failed" && (
        <Failure message={validation.message} kind={validation.kind} />
      )}
      {validation.state === "ready" && (
        <>
          <p className={validation.value.valid ? "verdict pass" : "verdict fail"}>
            {validation.value.valid
              ? `Pass — ${counted(validation.value.checked_projects, "project")} checked.`
              : `${counted(validation.value.blocking, "blocking issue")} — project details, search and permission changes are refused until they are fixed.`}
          </p>
          <ul className="issues">
            {validation.value.issues.map((issue, index) => (
              <li key={`${issue.code}-${issue.path}-${index}`} className={`issue ${issue.severity}`}>
                <div className="issue-head">
                  <span className="badge">{issue.severity === "error" ? "Error" : "Warning"}</span>
                  <span>{issue.message}</span>
                </div>
                <div className="muted small">
                  {issueWhere(issue)} · <code>{issue.code}</code> · <span className="path">{issue.path}</span>
                </div>
                <div className="fix">{fixText(issue.fix)}</div>
              </li>
            ))}
          </ul>
        </>
      )}
    </section>
  );
}
