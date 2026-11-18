// The read-only screens: projects, a project's details, validation and search, and which tabs
// the whole workspace and a project each show.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
// Screenshots are for people, so they are taken at twice the pixel density.
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

await page.getByRole("button", { name: "security" }).waitFor();
check("the sidebar lists the projects", (await page.getByRole("navigation").textContent()).includes("demo-chat"));
const tabNames = async () => (await page.getByRole("tab").allTextContents()).map((t) => t.trim());
check("Whole workspace comes first, above the projects", (await page.locator(".sidebar nav .item").first().textContent()) === "Whole workspace");
check("the sidebar no longer shows the folder path", !(await page.locator(".sidebar").textContent()).includes("Chosen by"));
check("the whole workspace has its own tabs", JSON.stringify(await tabNames()) === JSON.stringify(["Overview", "Validation", "Search", "Configuration"]), JSON.stringify(await tabNames()));
check("the whole workspace's Overview says where the folder came from", (await page.locator(".content").textContent()).includes("Chosen by"));

await page.getByRole("tab", { name: "Validation" }).click();
await page.locator(".verdict").waitFor();
const validation = await page.locator(".content").textContent();
check("a failing validation says what blocks", validation.includes("1 blocking issue"));
check("each issue says how to fix it, without naming a command", validation.includes("Fix project.toml, then validate again.") && !validation.includes("ai-workspace validate"));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/validation.png` });

await page.getByRole("button", { name: "security" }).click();
check("a project has its own tabs, chat first", JSON.stringify(await tabNames()) === JSON.stringify(["Chat", "Overview", "Search", "Permissions"]), JSON.stringify(await tabNames()));
check("a project opens on its chat", (await page.getByRole("tab", { name: "Chat" }).getAttribute("aria-selected")) === "true");
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("heading", { name: "Security" }).waitFor();
const overview = await page.locator(".content").textContent();
check("a project's description and model are shown", overview.includes("Router setup") && overview.includes("sec"));
const panel = await page.locator(".content .panel").first().boundingBox();
const content = await page.locator(".content").boundingBox();
check("the tab's page is centred", Math.abs(panel.x - content.x - (content.x + content.width - panel.x - panel.width)) < 30, JSON.stringify({ panel, content }));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/overview.png` });

await page.getByRole("tab", { name: "Search" }).click();
await page.getByLabel("Search terms").fill("firewall");
await page.getByRole("button", { name: "Search", exact: true }).click();
await page.locator(".hits li").first().waitFor();
check("search shows ranked hits", (await page.locator(".hits li").count()) === 2);
const searchCall = await page.evaluate(() => window.__calls.find((c) => c.command === "workspace_search"));
check("search is scoped to the selected project", searchCall?.args?.project === "security", JSON.stringify(searchCall));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/search.png` });

await finish(browser);
