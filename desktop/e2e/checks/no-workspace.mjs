// A folder without a workspace is described, not created (issue #55): the app says so and
// offers Initialize; nothing is written until the person presses it.

import { check, finish, FIXTURE, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 700 }, deviceScaleFactor: SHOTS ? 2 : 1 });
const empty = {
  ...FIXTURE,
  status: {
    ...FIXTURE.status,
    initialized: false,
    root: "/tmp/aiw-env-test",
    root_source: "environment variable AI_WORKSPACE_HOME",
    projects: [],
    archived: [],
  },
};
await stubEngine(page, { fixture: empty });
await page.goto(URL);

const notice = page.getByRole("status", { name: "No workspace" });
await notice.waitFor();
check("the overview says the folder has no workspace yet", (await notice.textContent()).includes("no workspace in this folder yet"));
check("and shows the folder and where it was chosen from", (await page.locator(".facts").textContent()).includes("/tmp/aiw-env-test"));
check("the sidebar says so too", await page.getByText("No workspace in this folder yet.").isVisible());
check("no New project before Initialize", (await page.getByRole("button", { name: /New project/ }).count()) === 0);
check("nothing is written on opening", !(await page.evaluate(() => window.__calls.some((c) => c.command === "workspace_init"))));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/no-workspace.png` });

await page.getByRole("button", { name: "Initialize the workspace" }).click();
await page.getByRole("heading", { name: "Workspace", exact: true }).waitFor();
await page.waitForFunction(() => !document.querySelector('[aria-label="No workspace"]'));
check("Initialize creates it, and the overview shows the workspace", (await page.locator(".facts").textContent()).includes("Projects folder"));
check("New project appears once there is a workspace", await page.getByRole("button", { name: /New project/ }).isVisible());

await finish(browser);
