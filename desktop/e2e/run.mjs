// The browser checks: the interface in a real Chromium, with a stub engine behind `invoke`,
// served by a Vite server of its own on a free port — so a `tauri dev` already running is no
// obstacle. Ollama, the shell and a workspace are never needed.
//
//   npm run e2e               every check in e2e/checks
//   npm run e2e -- chat       the ones whose file names contain "chat"
//
// AIW_E2E_CHROMIUM names a Chromium to use instead of Playwright's own.

import { spawn } from "node:child_process";
import { readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";

const HERE = dirname(fileURLToPath(import.meta.url));
const CHECKS = join(HERE, "checks");
const TIMEOUT_MS = 120_000;

const wanted = process.argv.slice(2);
const files = readdirSync(CHECKS)
  .filter((name) => name.endsWith(".mjs"))
  .filter((name) => wanted.length === 0 || wanted.some((part) => name.includes(part)))
  .sort();
if (files.length === 0) {
  console.error(`No check in e2e/checks matches ${wanted.join(", ")}.`);
  process.exit(1);
}

const server = await createServer({
  root: join(HERE, ".."),
  clearScreen: false,
  logLevel: "warn",
  server: { port: 0, strictPort: false, open: false },
});
await server.listen();
const url = server.resolvedUrls.local[0];

function run(file) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [join(CHECKS, file)], {
      env: { ...process.env, URL: url },
      stdio: ["ignore", "inherit", "inherit"],
    });
    const timer = setTimeout(() => {
      console.error(`${file}: stopped, still running after ${TIMEOUT_MS / 1000} s`);
      child.kill("SIGTERM");
    }, TIMEOUT_MS);
    child.on("close", (code) => {
      clearTimeout(timer);
      resolve(code === 0);
    });
  });
}

let failedFiles = 0;
for (const file of files) {
  console.log(`\n== ${file}`);
  if (!(await run(file))) failedFiles += 1;
}
await server.close();

console.log(failedFiles === 0 ? "\nAll browser checks passed." : `\n${failedFiles} check file(s) failed.`);
process.exit(failedFiles === 0 ? 0 : 1);
