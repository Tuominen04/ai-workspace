// The write operations: create a project, grant and revoke, choose the workspace folder,
// initialize from a validation issue.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();

{
  const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
  await stubEngine(page);
  await page.goto(URL);

  // Create a project.
  await page.getByRole("button", { name: "+ New project" }).click();
  await page.getByLabel("Project name").fill("Config");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await page.getByRole("alert").waitFor();
  check("a refused project name is explained", (await page.getByRole("alert").textContent()).includes("invalid project name"));
  await page.getByLabel("Project name").fill("Home Lab");
  await page.getByLabel("Project description").fill("Switches and VLANs");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await page.getByRole("heading", { name: "Home Lab" }).waitFor();
  check("a new project appears and is selected", (await page.getByRole("button", { name: "home-lab" }).getAttribute("class")).includes("active"));

  // Grant and revoke.
  await page.getByRole("button", { name: "security" }).click();
  await page.getByRole("tab", { name: "Permissions" }).click();
  await page.getByText("2 permissions").waitFor();
  check("the default grants are listed", (await page.locator(".grant").count()) === 2);
  await page.getByLabel("Path to grant").fill("/home/me/missing");
  await page.getByRole("button", { name: "Grant" }).click();
  await page.getByRole("alert").waitFor();
  check("a path that does not exist is refused", (await page.getByRole("alert").textContent()).includes("invalid permission path"));
  await page.getByLabel("Path to grant").fill("/home/me/router-backups");
  await page.getByRole("button", { name: "Grant" }).click();
  await page.getByText("3 permissions").waitFor();
  check("a granted path is listed as temporary", (await page.locator(".grant.active", { hasText: "/home/me/router-backups" }).textContent()).includes("temporary"));
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/permissions.png` });
  await page.getByRole("button", { name: "Revoke /home/me/router-backups" }).click();
  await page.getByText("2 permissions").waitFor();
  check("revoking removes it", (await page.locator(".notice").textContent()).includes("Revoked /home/me/router-backups"));
  check("default grants cannot be revoked", (await page.getByRole("button", { name: /^Revoke/ }).count()) === 0);

  // Choose the workspace folder.
  await page.getByRole("tab", { name: "Configuration" }).click();
  await page.getByLabel("New workspace folder").fill("/home/me/nope");
  await page.getByRole("button", { name: "Use this folder" }).click();
  await page.getByRole("alert").waitFor();
  check("a missing folder is refused", (await page.getByRole("alert").textContent()).includes("does not exist"));
  await page.getByLabel("New workspace folder").fill("/home/me/ai-local/workspaces");
  await page.getByRole("button", { name: "Use this folder" }).click();
  await page.getByRole("status").waitFor();
  check("a saved folder is confirmed", (await page.getByRole("status").textContent()).includes("/home/me/ai-local/workspaces"));
  const statusCalls = await page.evaluate(() => window.__calls.filter((c) => c.command === "workspace_status").length);
  check("the workspace is read again after the change", statusCalls >= 3, `status read ${statusCalls} times`);
  await page.close();
}

{
  // Initialize from a validation issue whose fix is to initialize.
  const page = await browser.newPage();
  const { FIXTURE } = await import("../support.mjs");
  const fixture = structuredClone(FIXTURE);
  fixture.validation.issues = [
    {
      severity: "error",
      scope: "workspace",
      subject: "configuration",
      path: "/home/me/AI/Config/workspace.toml",
      code: "WS-CONFIG-MISSING",
      message: "workspace configuration file is missing",
      fix: { kind: "run", action: "init", purpose: "bootstrap default workspace configuration" },
    },
  ];
  await stubEngine(page, { fixture });
  await page.goto(URL);
  await page.getByRole("tab", { name: "Validation" }).click();
  await page.getByRole("button", { name: "Initialize the workspace" }).click();
  await page.waitForFunction(() => window.__calls.some((c) => c.command === "workspace_init"));
  const after = await page.evaluate(() => window.__calls.map((c) => c.command));
  check("initializing runs, then validation runs again", after.lastIndexOf("workspace_validate") > after.indexOf("workspace_init"), after.join(","));
  await page.close();
}

await finish(browser);
