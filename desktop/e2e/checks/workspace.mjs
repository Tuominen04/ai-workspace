// The read-only screens: projects, a project's details, validation and search.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
// Screenshots are for people, so they are taken at twice the pixel density.
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

await page.getByRole("button", { name: "security" }).waitFor();
check("the sidebar lists the projects", (await page.getByRole("navigation").textContent()).includes("demo-chat"));
check(
  "the sidebar says where the workspace folder came from",
  (await page.locator(".sidebar").textContent()).includes("Chosen by user config file"),
);

await page.getByRole("button", { name: "security" }).click();
await page.getByRole("heading", { name: "Security" }).waitFor();
const overview = await page.locator(".content").textContent();
check("a project's description and model are shown", overview.includes("Router setup") && overview.includes("sec"));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/overview.png` });

await page.getByRole("tab", { name: "Validation" }).click();
await page.locator(".verdict").waitFor();
const validation = await page.locator(".content").textContent();
check("a failing validation says what blocks", validation.includes("1 blocking issue"));
check("each issue says how to fix it, without naming a command", validation.includes("Fix project.toml, then validate again.") && !validation.includes("ai-workspace validate"));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/validation.png` });

await page.getByRole("tab", { name: "Search" }).click();
await page.getByLabel("Search terms").fill("firewall");
await page.getByRole("button", { name: "Search", exact: true }).click();
await page.locator(".hits li").first().waitFor();
check("search shows ranked hits", (await page.locator(".hits li").count()) === 2);
const searchCall = await page.evaluate(() => window.__calls.find((c) => c.command === "workspace_search"));
check("search is scoped to the selected project", searchCall?.args?.project === "security", JSON.stringify(searchCall));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/search.png` });

await finish(browser);
