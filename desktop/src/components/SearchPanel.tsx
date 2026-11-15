import { useState, type FormEvent } from "react";

import { engine, messageOf, type SearchResults } from "../lib/engine";
import { counted, fileName } from "../lib/words";
import { Failure } from "./Failure";

type State =
  | { state: "idle" }
  | { state: "searching" }
  | { state: "done"; results: SearchResults }
  | { state: "failed"; message: string; kind: string | null };

/** Ranked search over what may be read: Base, and the selected project's Knowledge and Session. */
export function SearchPanel({ project }: { project: string | null }) {
  const [query, setQuery] = useState("");
  const [state, setState] = useState<State>({ state: "idle" });

  async function submit(event: FormEvent) {
    event.preventDefault();
    if (query.trim() === "") return;
    setState({ state: "searching" });
    try {
      setState({ state: "done", results: await engine.workspace.search(query, project) });
    } catch (thrown) {
      setState({
        state: "failed",
        message: messageOf(thrown),
        kind: (thrown as { kind?: string }).kind ?? null,
      });
    }
  }

  return (
    <section className="panel">
      <h2>Search</h2>
      <form className="search" onSubmit={submit}>
        <input
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={project ? `Search Base and ${project}` : "Search Base knowledge"}
          aria-label="Search terms"
        />
        <button type="submit" disabled={query.trim() === "" || state.state === "searching"}>
          Search
        </button>
      </form>
      <p className="muted small">
        {project
          ? `Looking in Base knowledge and project ${project} (its Knowledge and Session folders).`
          : "Looking in Base knowledge only. Choose a project to include its files."}
      </p>

      {state.state === "searching" && <p className="muted">Searching…</p>}
      {state.state === "failed" && <Failure message={state.message} kind={state.kind} />}
      {state.state === "done" && (
        <>
          <p className="result-count">
            {state.results.hits.length === 0
              ? `Nothing matched “${state.results.query}”.`
              : `${counted(state.results.hits.length, "result")} for “${state.results.query}”`}
          </p>
          <ol className="hits">
            {state.results.hits.map((hit) => (
              <li key={hit.path}>
                <div className="hit-head">
                  <span className={`kind ${hit.source_kind}`}>{hit.source_kind}</span>
                  <span className="hit-name" title={hit.path}>
                    {fileName(hit.path)}
                  </span>
                  <span className="muted small">score {hit.score.toFixed(1)}</span>
                </div>
                <p className="snippet">{hit.snippet}</p>
                <div className="path muted small">{hit.path}</div>
              </li>
            ))}
          </ol>
        </>
      )}
    </section>
  );
}
