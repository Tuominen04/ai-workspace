// The Configuration tab: what workspace.toml says, and a missing file explained.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();

{
  const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
  await stubEngine(page);
  await page.goto(URL);
  await page.getByRole("tab", { name: "Configuration" }).click();
  await page.getByRole("heading", { name: "Configuration" }).waitFor();
  await page.locator(".facts").first().waitFor();
  const text = await page.locator(".content").textContent();
  check("the configuration file and default model are shown", text.includes("/home/me/AI/Config/workspace.toml") && text.includes("qwen3.6:latest"));
  check("new project folders are listed", text.includes("Knowledge, Specs, Resources, Session"));
  check("the folders recorded in the file are shown apart", text.includes("Recorded in the file"));
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/configuration.png` });
  await page.close();
}

{
  const page = await browser.newPage();
  await stubEngine(page, {
    failures: { workspace_config: { kind: "ConfigMissing", message: "workspace configuration file is missing: /w/Config/workspace.toml" } },
  });
  await page.goto(URL);
  await page.getByRole("tab", { name: "Configuration" }).click();
  await page.getByRole("alert").waitFor();
  check("a missing file is explained", (await page.locator(".content").textContent()).includes("has no configuration file yet"));
  await page.getByRole("button", { name: "Initialize the workspace" }).click();
  await page.waitForFunction(() => window.__calls.some((c) => c.command === "workspace_init"));
  check("Initialize runs from the Configuration tab", true);
  await page.close();
}

await finish(browser);
