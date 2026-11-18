// Archiving and restoring projects (ADR-0015): the Archived list, a read-only archived
// project, Restore, Archive ending an open chat session, and a taken name refused.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

const archivedList = page.getByRole("navigation", { name: "Archived projects" });
await archivedList.waitFor();
check("archived projects are listed apart", (await archivedList.textContent()).includes("old-lab"));
const listBox = await archivedList.boundingBox();
const themeBox = await page.getByLabel("Theme").boundingBox();
check("the Archived list sits at the bottom, above the theme", listBox.y > 400 && listBox.y < themeBox.y, JSON.stringify({ listBox, themeBox }));

// An archived project is read-only.
await archivedList.getByRole("button", { name: "old-lab" }).click();
const tabs = (await page.getByRole("tab").allTextContents()).map((t) => t.trim());
check("an archived project shows only its overview", JSON.stringify(tabs) === JSON.stringify(["Overview"]), JSON.stringify(tabs));
check("it says it is archived and offers Restore", (await page.locator(".content").textContent()).includes("Archived.") && (await page.getByRole("button", { name: "Restore" }).isVisible()));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/archived.png` });

// Restore: back among the projects, opened on its chat.
await page.getByRole("button", { name: "Restore" }).click();
await page.getByRole("button", { name: "Start session in old-lab" }).waitFor();
check("a restored project is a normal project again, on its chat", (await page.getByRole("tab", { name: "Chat" }).getAttribute("aria-selected")) === "true");
check("and the Archived list is gone when empty", (await archivedList.count()) === 0);

// Archive with a chat session open: the session ends first, and the whole workspace shows.
await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("button", { name: "Archive" }).click();
await archivedList.waitFor();
check("archiving moves the project to the Archived list", (await archivedList.textContent()).includes("security"));
check("and out of the projects", (await page.getByRole("navigation", { name: "Projects", exact: true }).getByRole("button", { name: "security", exact: true }).count()) === 0);
check("the whole workspace is selected afterwards", (await page.getByRole("button", { name: "Whole workspace" }).getAttribute("class")).includes("active"));
check("the open chat session in it was ended", (await page.locator(".sidebar .dot").count()) === 0);

// A taken name is refused with "Already exists".
await page.getByRole("button", { name: "+ New project" }).click();
await page.getByLabel("Project name").fill("Security");
await page.getByRole("button", { name: "Create", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).waitFor();
await archivedList.getByRole("button", { name: "security" }).click();
await page.getByRole("button", { name: "Restore" }).click();
await page.getByRole("alert").waitFor();
check("restoring over an existing project is refused", (await page.getByRole("alert").textContent()).includes("Already exists"));

await finish(browser);
