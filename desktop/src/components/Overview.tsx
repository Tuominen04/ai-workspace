import { engine, type WorkspaceStatus } from "../lib/engine";
import { useLoad } from "../lib/useLoad";
import { counted, dateText } from "../lib/words";
import { Failure } from "./Failure";

/** The selected project's details, or the workspace's folders when none is selected. */
export function Overview({ status, project }: { status: WorkspaceStatus; project: string | null }) {
  if (project === null) {
    return (
      <section className="panel">
        <h2>Workspace</h2>
        <dl className="facts">
          <dt>Folder</dt>
          <dd className="path">{status.root}</dd>
          <dt>Chosen by</dt>
          <dd>{status.root_source}</dd>
          <dt>Base knowledge</dt>
          <dd className="path">{status.base}</dd>
          <dt>Projects folder</dt>
          <dd className="path">{status.projects_dir}</dd>
          <dt>Projects</dt>
          <dd>{counted(status.projects.length, "project")}</dd>
        </dl>
      </section>
    );
  }
  return <ProjectOverview name={project} />;
}

function ProjectOverview({ name }: { name: string }) {
  const [details] = useLoad(() => engine.project.show(name), name);

  if (details.state === "loading") return <p className="muted panel">Loading {name}…</p>;
  if (details.state === "failed")
    return (
      <section className="panel">
        <h2>{name}</h2>
        <Failure message={details.message} kind={details.kind} />
      </section>
    );

  const p = details.value;
  return (
    <section className="panel">
      <h2>{p.display_name}</h2>
      <p className="description">{p.description || <span className="muted">No description.</span>}</p>
      <dl className="facts">
        <dt>Folder</dt>
        <dd className="path">{p.path}</dd>
        <dt>Model</dt>
        <dd>{p.model ?? <span className="muted">workspace default</span>}</dd>
        <dt>Created</dt>
        <dd>{dateText(p.created_at)}</dd>
        <dt>Knowledge</dt>
        <dd>{p.knowledge_dir}</dd>
        <dt>Specs</dt>
        <dd>{p.specs_dir}</dd>
        <dt>Resources</dt>
        <dd>{p.resources_dir}</dd>
        <dt>Session</dt>
        <dd>{p.session_dir}</dd>
      </dl>
    </section>
  );
}
