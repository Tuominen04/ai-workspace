// What every check shares: where the interface is, a browser, a stub engine, and the tally
// that decides the exit code. A check can also run on its own against `npm run dev`:
//
//   URL=http://localhost:5174/ node e2e/checks/workspace.mjs [screenshot-folder]

import { chromium } from "playwright";

export const URL = process.env.URL ?? "http://localhost:5174/";

/** Where to save screenshots, if a folder was given; never under `npm run e2e`. */
export const SHOTS = process.argv[2] ?? null;

export function launch() {
  return chromium.launch({ executablePath: process.env.AIW_E2E_CHROMIUM || undefined });
}

/** A fixed, believable workspace for the stub to answer from. */
export const FIXTURE = {
  status: {
    initialized: true,
    root: "/home/me/AI",
    root_source: "user config file /home/me/.config/ai-workspace/config.toml",
    base: "/home/me/AI/Base",
    projects_dir: "/home/me/AI/Projects",
    projects: [
      { name: "security", path: "/home/me/AI/Projects/security" },
      { name: "demo-chat", path: "/home/me/AI/Projects/demo-chat" },
    ],
    archived: [{ name: "old-lab", path: "/home/me/AI/Projects/Archive/old-lab" }],
  },
  details: {
    security: {
      name: "security",
      path: "/home/me/AI/Projects/security",
      display_name: "Security",
      description: "Router setup, firewall rules and hardening notes",
      created_at: "1794690257",
      model: "sec",
      knowledge_dir: "Knowledge",
      specs_dir: "Specs",
      resources_dir: "Resources",
      session_dir: "Session",
    },
  },
  validation: {
    valid: false,
    checked_projects: 2,
    blocking: 1,
    issues: [
      {
        severity: "error",
        scope: "project",
        subject: "demo-chat",
        path: "/home/me/AI/Projects/demo-chat",
        code: "PROJECT-METADATA-INVALID",
        message: "project metadata could not be loaded: expected a value",
        fix: { kind: "fix_then_rerun", step: "fix project.toml", action: "validate" },
      },
    ],
  },
  search: {
    query: "firewall",
    hits: [
      {
        source_kind: "project",
        path: "/home/me/AI/Projects/security/Knowledge/router.md",
        score: 7.5,
        snippet: "The router firewall blocks inbound ports except 22 from the LAN…",
      },
      {
        source_kind: "base",
        path: "/home/me/AI/Base/Fedora/firewalld.md",
        score: 3,
        snippet: "firewalld zones: public for Wi-Fi, home for the wired LAN…",
      },
    ],
  },
  config: {
    active_root: "/home/me/AI",
    root_source: "user config file /home/me/.config/ai-workspace/config.toml",
    config_file: "/home/me/AI/Config/workspace.toml",
    default_model: "qwen3.6:latest",
    runtime: "the installed ollama command",
    project_metadata_file: "project.toml",
    default_project_directories: ["Knowledge", "Specs", "Resources", "Session"],
    recorded_root: "/home/me/AI",
    recorded_base: "/home/me/AI/Base",
    recorded_projects: "/home/me/AI/Projects",
    recorded_cache: "/home/me/AI/Cache",
    recorded_logs: "/home/me/AI/Logs",
    recorded_config: "/home/me/AI/Config",
  },
  session: {
    project: "security",
    can_chat: true,
    runtime: "Ollama is already running on 127.0.0.1:11434",
    model: "sec",
    agent_included: true,
    agent: "the model's own instructions are kept",
    base_files: ["/home/me/AI/Base/Fedora/firewalld.md"],
    project_files: [
      "/home/me/AI/Projects/security/Knowledge/router.md",
      "/home/me/AI/Projects/security/Knowledge/ssh.md",
    ],
    session_files: [],
    granted_files: ["/home/me/Documents/Security/router-hardening-notes.md"],
    skipped_files: ["/home/me/AI/Projects/security/Knowledge/wifi-password.md"],
    budget: "12288 bytes of workspace text, from the model's 8192-token window",
    truncated_files: ["/home/me/AI/Projects/security/Knowledge/history.md (11800 of 26000 bytes shown)"],
  },
};

/**
 * Installs a stub of the desktop runtime before the page loads. Every call is recorded in
 * `window.__calls`. `failures` maps a command to the `{ kind, message }` it should reject with.
 */
export async function stubEngine(page, { fixture = FIXTURE, failures = {} } = {}) {
  await page.addInitScript(
    ({ fixture, failures }) => {
      window.__calls = [];
      // A little state, so writes show up in later reads.
      const projects = [...fixture.status.projects];
      const archived = [...(fixture.status.archived ?? [])];
      const grants = [
        { path: "/home/me/AI/Base", category: "default", status: "active", granted_at: "1794690257" },
        { path: "/home/me/AI/Projects/security", category: "default", status: "active", granted_at: "1794690257" },
      ];
      // The open session as the shell reports it now: its context lists the active grants.
      const sessionNow = () => ({
        ...fixture.session,
        project: window.__session ?? fixture.session.project,
        granted_files: grants.filter((g) => g.category === "temporary" && g.status === "active").map((g) => g.path),
      });
      const answers = {
        // A folder without a workspace (fixture.status.initialized false) becomes one on init.
        workspace_status: () => ({
          ...fixture.status,
          initialized: window.__initialized ?? fixture.status.initialized,
          projects: [...projects],
          archived: [...archived],
        }),
        // Archive and restore move a project between the two lists; a taken name is refused.
        project_archive: ({ name }) => {
          if (archived.some((p) => p.name === name))
            return Promise.reject({ kind: "AlreadyExists", message: `Already exists: /home/me/AI/Projects/Archive/${name}` });
          const index = projects.findIndex((p) => p.name === name);
          const path = `/home/me/AI/Projects/Archive/${name}`;
          projects.splice(index, 1);
          archived.push({ name, path });
          const closed = window.__session === name ? { project: name, revoked: 0 } : null;
          if (closed) window.__session = null;
          return { name, path, session_closed: closed };
        },
        // Delete moves to the Trash; "demo-chat" stands for a folder the Trash cannot take.
        project_delete: ({ name, archived: fromArchive, erase }) => {
          const list = fromArchive ? archived : projects;
          const path = `/home/me/AI/Projects/${fromArchive ? "Archive/" : ""}${name}`;
          if (name === "demo-chat" && !erase)
            return Promise.reject({ kind: "TrashUnavailable", message: `cannot move ${path} to the Trash: no trash folder on this drive; nothing was deleted` });
          list.splice(list.findIndex((p) => p.name === name), 1);
          const closed = window.__session === name ? { project: name, revoked: 0 } : null;
          if (closed) window.__session = null;
          return { name, path, erased: Boolean(erase && name === "demo-chat"), session_closed: closed };
        },
        // Rename: the folder follows the new name; a taken folder or a bad name is refused.
        project_rename: ({ name, newName }) => {
          const trimmed = newName.trim();
          if (!/^[A-Za-z0-9 _-]+$/.test(trimmed) || trimmed.toLowerCase() === "archive")
            return Promise.reject({ kind: "InvalidProjectName", message: `invalid project name '${newName}'` });
          const slug = trimmed.toLowerCase().split(/\s+/).join("-");
          if (slug !== name && projects.some((p) => p.name === slug))
            return Promise.reject({ kind: "AlreadyExists", message: `Already exists: /home/me/AI/Projects/${slug}` });
          const path = `/home/me/AI/Projects/${slug}`;
          projects.splice(projects.findIndex((p) => p.name === name), 1, { name: slug, path });
          fixture.details[slug] = { ...(fixture.details[name] ?? fixture.details.security), name: slug, path, display_name: trimmed };
          const closed = window.__session === name ? { project: name, revoked: 0 } : null;
          if (closed) window.__session = null;
          return { project: fixture.details[slug], session_closed: closed };
        },
        project_clone: ({ name }) => {
          const slug = `${name}-clone`;
          if (projects.some((p) => p.name === slug))
            return Promise.reject({ kind: "AlreadyExists", message: `Already exists: /home/me/AI/Projects/${slug}` });
          const path = `/home/me/AI/Projects/${slug}`;
          projects.push({ name: slug, path });
          const source = fixture.details[name] ?? fixture.details.security;
          fixture.details[slug] = { ...source, name: slug, path, display_name: `${source.display_name} (Clone)` };
          return fixture.details[slug];
        },
        project_restore: ({ name }) => {
          if (projects.some((p) => p.name === name))
            return Promise.reject({ kind: "AlreadyExists", message: `Already exists: /home/me/AI/Projects/${name}` });
          const index = archived.findIndex((p) => p.name === name);
          const path = `/home/me/AI/Projects/${name}`;
          archived.splice(index, 1);
          projects.push({ name, path });
          fixture.details[name] ??= { ...fixture.details.security, name, display_name: name, path };
          return { name, path, session_closed: null };
        },
        workspace_init: () => {
          window.__initialized = true;
          return { root: fixture.status.root, config_file: fixture.config.config_file, config_created: true };
        },
        workspace_set_root: ({ path }) => {
          if (!path.includes("nope")) window.__savedRoot = path;
          return path.includes("nope")
            ? Promise.reject({ kind: "InvalidWorkspaceRoot", message: `cannot use '${path}' as the workspace folder: it does not exist` })
            : {
                root: path,
                config_file: "/home/me/.config/ai-workspace/config.toml",
                env_override: fixture.envOverride ?? null,
                session_closed: null,
              };
        },
        // While the variable decides (fixture.envOverride) and nobody switched: Switch now.
        workspace_switch_offer: () =>
          fixture.envOverride && !window.__switched
            ? {
                variable_root: fixture.envOverride,
                saved_root: window.__savedRoot ?? "/home/me/ai-local/workspaces",
                saved_source: "user config file /home/me/.config/ai-workspace/config.toml",
              }
            : null,
        workspace_switch_to_saved: () => {
          window.__switched = true;
          return {
          root: window.__savedRoot ?? "/home/me/ai-local/workspaces",
          root_source: "user config file /home/me/.config/ai-workspace/config.toml (AI_WORKSPACE_HOME ignored until the app restarts)",
          session_closed: null,
          };
        },
        project_create: ({ name, description }) => {
          if (name === "Config")
            return Promise.reject({ kind: "InvalidProjectName", message: "invalid project name 'Config'" });
          const slug = name.toLowerCase().replace(/\s+/g, "-");
          projects.push({ name: slug, path: `/home/me/AI/Projects/${slug}` });
          fixture.details[slug] = { ...fixture.details.security, name: slug, display_name: name, description, model: null, path: `/home/me/AI/Projects/${slug}` };
          return fixture.details[slug];
        },
        models_installed: () => ["qwen3.6:latest", "sec:latest"],
        project_set_model: ({ name, model }) => {
          if (model && /\s/.test(model))
            return Promise.reject({ kind: "InvalidModelName", message: `invalid model name '${model}'` });
          fixture.details[name] = { ...fixture.details[name], model: model ?? null };
          return fixture.details[name];
        },
        permission_list: ({ filter }) => {
          const shown = grants.filter((g) => !filter || g.path.toLowerCase().includes(filter.toLowerCase()));
          return { project: "security", session_id: "security-1", total: shown.length, page: 1, page_size: 25, grants: shown };
        },
        permission_grant: ({ path }) => {
          if (path.includes("missing"))
            return Promise.reject({ kind: "InvalidPermissionPath", message: `invalid permission path '${path}'` });
          const existing = grants.find((g) => g.path === path);
          if (!existing) grants.unshift({ path, category: "temporary", status: "active", granted_at: "1794690300" });
          return { project: "security", path, created: !existing };
        },
        permission_revoke: ({ path }) => {
          const index = grants.findIndex((g) => g.path === path && g.category === "temporary");
          if (index < 0) return Promise.reject({ kind: "PermissionGrantNotFound", message: `no active temporary permission found for '${path}'` });
          grants.splice(index, 1);
          return { project: "security", path };
        },
        workspace_config: () => fixture.config,
        workspace_validate: () => fixture.validation,
        project_show: ({ name }) =>
          fixture.details[name] ?? Promise.reject({ kind: "ProjectNotFound", message: `project '${name}' was not found` }),
        workspace_search: () => fixture.search,
        chat_open: ({ project }) => {
          window.__session = project;
          return { ...fixture.session, project };
        },
        chat_context: () => sessionNow(),
        chat_send: ({ message, onPiece }) => {
          // "flaky" fails once, like a timeout, then works.
          if (message.includes("flaky") && !window.__flakyFailed) {
            window.__flakyFailed = true;
            return Promise.reject({
              kind: "RuntimeRequestFailed",
              message: "request to the AI runtime failed: timed out reading response",
            });
          }
          const text = message.includes("rules")
            ? "## My Rules (short version)\n\n1. **Exact Fedora commands** – `dnf`, `systemctl`, `firewall-cmd`. Each one gets a one-line explanation.\n2. **Warn before destructive actions** – back up first, here's how to undo.\n3. **Smallest fix wins** – I won't suggest `setenforce 0`.\n\n---\n\n### What I *can't* do\n\n- See your filesystem, processes, or network.\n- Run commands for you.\n\nReady when you are. Paste a config, a `journalctl` snippet or an `ss -tlnp` dump."
            : `You asked: ${message}`;
          // Streamed in a few pieces, as Ollama does; slower for "slowly" so a check can watch.
          const size = Math.ceil(text.length / 4);
          const pieces = [0, 1, 2, 3].map((n) => text.slice(n * size, (n + 1) * size)).filter(Boolean);
          // "long chat" stands for a conversation that outgrew the model's window.
          if (message.includes("long chat")) window.__leftOut = (window.__leftOut ?? 0) + 2;
          const session = sessionNow();
          return stream(onPiece, pieces, message.includes("slowly") ? 300 : 30).then((reply) => ({
            text: reply,
            left_out_exchanges: window.__leftOut ?? 0,
            session,
          }));
        },
        chat_close: () => ({ project: fixture.session.project, revoked: 1 }),
        // Tauri's event listener, and the window close the shell would do.
        "plugin:event|listen": ({ event, handler }) => {
          (window.__listeners[event] ??= []).push(handler);
          return handler;
        },
        "plugin:event|unlisten": () => null,
        app_quit: () => {
          window.__quit = true;
          return null;
        },
        chat_draft_note: () => "# Router hardening\n- Discussed SSH exposure\n- Next: review the guest VLAN",
        note_save: ({ project, text }) =>
          text.trim() === ""
            ? Promise.reject({ kind: "Message", message: "a session note cannot be empty" })
            : { project, path: `/home/me/AI/Projects/${project}/Session/session-note-2026-11-15-160000.md` },
      };
      // Tauri's Channel registers a callback through transformCallback and the shell sends
      // { index, message } to it; the stub does the same for streamed replies.
      const callbacks = new Map();
      let nextCallback = 1;
      const stream = (channel, pieces, gap) =>
        new Promise((resolve) => {
          pieces.forEach((piece, index) =>
            setTimeout(() => callbacks.get(channel.id)?.({ index, message: piece }), gap * (index + 1)),
          );
          setTimeout(() => {
            callbacks.get(channel.id)?.({ index: pieces.length, end: true });
            resolve(pieces.join(""));
          }, gap * (pieces.length + 1));
        });
      // What the shell's close request would deliver: listeners by event name.
      window.__listeners = {};
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
      window.__emit = (event, payload) =>
        (window.__listeners[event] ?? []).forEach((id) => callbacks.get(id)?.({ event, id, payload }));
      window.__TAURI_INTERNALS__ = {
        invoke: async (command, args) => {
          window.__calls.push({ command, args });
          if (failures[command]) throw failures[command];
          const answer = answers[command];
          if (!answer) throw { kind: "Bridge", message: `the stub has no ${command}` };
          return answer(args ?? {});
        },
        transformCallback: (callback) => {
          const id = nextCallback++;
          callbacks.set(id, callback);
          return id;
        },
        unregisterCallback: (id) => callbacks.delete(id),
      };
    },
    { fixture, failures },
  );
}

let passed = 0;
let failed = 0;

/** One result, printed as it happens. Returns `ok`, so a check can guard the ones after it. */
export function check(name, ok, detail = "") {
  console.log(`${ok ? "ok  " : "FAIL"} ${name}${ok || !detail ? "" : ` — ${detail}`}`);
  if (ok) passed += 1;
  else failed += 1;
  return ok;
}

/** Ends the check file with an exit code from the tally. */
export async function finish(browser) {
  await browser.close();
  console.log(`${passed} passed, ${failed} failed`);
  process.exit(failed === 0 ? 0 : 1);
}
