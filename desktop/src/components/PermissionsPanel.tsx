import { useState, type FormEvent } from "react";

import { engine } from "../lib/engine";
import { failureOf, type FailureInfo } from "../lib/failure";
import { useLoad } from "../lib/useLoad";
import { counted, dateText } from "../lib/words";
import { Failure } from "./Failure";

/**
 * A project's permission session: the default grants (Base and the project) and temporary ones
 * with their status. Grant a path, revoke a temporary grant. Ending a chat session in this
 * project revokes the temporary grants too.
 */
export function PermissionsPanel({ project }: { project: string | null }) {
  if (project === null) {
    return (
      <section className="panel">
        <h2>Permissions</h2>
        <p className="muted">Choose a project on the left to see and change its permissions.</p>
      </section>
    );
  }
  // Keyed so that switching projects starts from a clean filter, page and form.
  return <ProjectPermissions key={project} project={project} />;
}

function ProjectPermissions({ project }: { project: string }) {
  const [filter, setFilter] = useState("");
  const [page, setPage] = useState(1);
  const [listing, reload] = useLoad(
    () => engine.permissions.list(project, filter, page),
    `${project}|${filter}|${page}`,
  );
  const [path, setPath] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [failure, setFailure] = useState<FailureInfo | null>(null);

  async function grant(event: FormEvent) {
    event.preventDefault();
    if (path.trim() === "" || busy) return;
    setBusy(true);
    setNotice(null);
    setFailure(null);
    try {
      const done = await engine.permissions.grant(project, path);
      setNotice(done.created ? `Granted ${done.path}.` : `${done.path} was already granted.`);
      setPath("");
      reload();
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  async function revoke(grantPath: string) {
    setBusy(true);
    setNotice(null);
    setFailure(null);
    try {
      const done = await engine.permissions.revoke(project, grantPath);
      setNotice(`Revoked ${done.path}.`);
      reload();
    } catch (thrown) {
      setFailure(failureOf(thrown));
    } finally {
      setBusy(false);
    }
  }

  const pages =
    listing.state === "ready"
      ? Math.max(1, Math.ceil(listing.value.total / Math.max(1, listing.value.page_size)))
      : 1;

  return (
    <section className="panel">
      <h2>Permissions · {project}</h2>
      <p className="muted small">
        The model only ever reads Base and this project, plus folders granted here. Temporary
        grants end when a chat session in this project ends.
      </p>

      <form className="search" onSubmit={grant} aria-label="Grant access">
        <input
          value={path}
          onChange={(event) => setPath(event.target.value)}
          placeholder="Folder or file to grant, e.g. /home/me/router-backups"
          aria-label="Path to grant"
        />
        <button type="submit" disabled={busy || path.trim() === ""}>
          Grant
        </button>
      </form>
      {notice && <p className="notice">{notice}</p>}
      {failure && <Failure message={failure.message} kind={failure.kind} />}

      <div className="panel-head">
        <input
          type="search"
          value={filter}
          onChange={(event) => {
            setFilter(event.target.value);
            setPage(1);
          }}
          placeholder="Filter by path"
          aria-label="Filter permissions"
        />
      </div>

      {listing.state === "loading" && <p className="muted">Reading permissions…</p>}
      {listing.state === "failed" && <Failure message={listing.message} kind={listing.kind} />}
      {listing.state === "ready" && (
        <>
          <p className="result-count">{counted(listing.value.total, "permission")}</p>
          <ul className="grants">
            {listing.value.grants.map((grant) => (
              <li key={grant.path + grant.category} className={`grant ${grant.status}`}>
                <span className={`kind ${grant.category}`}>{grant.category}</span>
                <span className="path grant-path" title={grant.path}>
                  {grant.path}
                </span>
                {grant.status === "unavailable" && <span className="badge warn">missing</span>}
                <span className="muted small">{dateText(grant.granted_at)}</span>
                {grant.category === "temporary" ? (
                  <button
                    type="button"
                    className="quiet"
                    onClick={() => revoke(grant.path)}
                    disabled={busy}
                    aria-label={`Revoke ${grant.path}`}
                  >
                    Revoke
                  </button>
                ) : (
                  <span className="muted small fixed">always</span>
                )}
              </li>
            ))}
          </ul>
          {pages > 1 && (
            <div className="row pager">
              <button type="button" className="quiet" disabled={page <= 1} onClick={() => setPage(page - 1)}>
                Previous
              </button>
              <span className="muted small">
                Page {listing.value.page} of {pages}
              </span>
              <button type="button" className="quiet" disabled={page >= pages} onClick={() => setPage(page + 1)}>
                Next
              </button>
            </div>
          )}
        </>
      )}
    </section>
  );
}
