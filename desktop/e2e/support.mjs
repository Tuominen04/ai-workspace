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
    root: "/home/me/AI",
    root_source: "user config file /home/me/.config/ai-workspace/config.toml",
    base: "/home/me/AI/Base",
    projects_dir: "/home/me/AI/Projects",
    projects: [
      { name: "security", path: "/home/me/AI/Projects/security" },
      { name: "demo-chat", path: "/home/me/AI/Projects/demo-chat" },
    ],
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
  session: {
    project: "security",
    can_chat: true,
    runtime: "Ollama is already running on 127.0.0.1:11434",
    base_files: ["/home/me/AI/Base/Fedora/firewalld.md"],
    project_files: [
      "/home/me/AI/Projects/security/Knowledge/router.md",
      "/home/me/AI/Projects/security/Knowledge/ssh.md",
    ],
    session_files: [],
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
      const answers = {
        workspace_status: () => fixture.status,
        workspace_validate: () => fixture.validation,
        project_show: ({ name }) =>
          fixture.details[name] ?? Promise.reject({ kind: "ProjectNotFound", message: `project '${name}' was not found` }),
        workspace_search: () => fixture.search,
        chat_open: ({ project }) => ({ ...fixture.session, project }),
        chat_send: ({ message }) =>
          new Promise((resolve) => setTimeout(() => resolve(`You asked: ${message}`), 150)),
        chat_close: () => ({ project: fixture.session.project, revoked: 1 }),
      };
      window.__TAURI_INTERNALS__ = {
        invoke: async (command, args) => {
          window.__calls.push({ command, args });
          if (failures[command]) throw failures[command];
          const answer = answers[command];
          if (!answer) throw { kind: "Bridge", message: `the stub has no ${command}` };
          return answer(args ?? {});
        },
        transformCallback: () => 0,
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
