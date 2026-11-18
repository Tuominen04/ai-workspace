// Archiving and restoring projects (ADR-0015): the Archived view in the sidebar, a read-only
// archived project, Restore, Archive ending an open chat session, the right-click menu, and a
// taken name refused.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

const projectsNav = page.getByRole("navigation", { name: "Projects", exact: true });
const archivedNav = page.getByRole("navigation", { name: "Archived projects" });
const toggle = page.getByRole("button", { name: /^Archived \(\d+\)$/ });

// The archived projects are one button at the bottom, not a growing list.
await toggle.waitFor();
check("the bottom of the sidebar counts the archived projects", (await toggle.textContent()) === "Archived (1)");
check("archived projects are not listed with the projects", !(await projectsNav.textContent()).includes("old-lab"));
const toggleBox = await toggle.boundingBox();
const themeBox = await page.getByLabel("Theme").boundingBox();
check("the Archived button sits at the bottom, above the theme", toggleBox.y > 400 && toggleBox.y < themeBox.y, JSON.stringify({ toggleBox, themeBox }));

// It swaps the sidebar to the archived projects.
await toggle.click();
await archivedNav.waitFor();
check("the archived view replaces the project list", (await projectsNav.count()) === 0 && (await archivedNav.textContent()).includes("old-lab"));

// An archived project is read-only.
await archivedNav.getByRole("button", { name: "old-lab" }).click();
const tabs = (await page.getByRole("tab").allTextContents()).map((t) => t.trim());
check("an archived project shows only its overview", JSON.stringify(tabs) === JSON.stringify(["Overview"]), JSON.stringify(tabs));
check("it says it is archived and offers Restore", (await page.locator(".content").textContent()).includes("Archived.") && (await page.getByRole("button", { name: "Restore" }).isVisible()));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/archived.png` });

// Back to the projects.
await page.getByRole("button", { name: "← Projects" }).click();
await projectsNav.waitFor();
check("← Projects goes back to the project list", (await archivedNav.count()) === 0);

// Restore from the overview: back among the projects, opened on its chat.
await toggle.click();
await archivedNav.getByRole("button", { name: "old-lab" }).click();
await page.getByRole("button", { name: "Restore" }).click();
await page.getByRole("button", { name: "Start session in old-lab" }).waitFor();
check("a restored project is a normal project again, on its chat", (await page.getByRole("tab", { name: "Chat" }).getAttribute("aria-selected")) === "true");
check("the sidebar shows the projects again, and no Archived button when none are left", (await projectsNav.count()) === 1 && (await toggle.count()) === 0);

// Archive from the Overview with a chat session open: the session ends first.
await projectsNav.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("button", { name: "Archive" }).click();
await toggle.waitFor();
check("archiving moves the project out of the projects", (await projectsNav.getByRole("button", { name: "security", exact: true }).count()) === 0);
check("the whole workspace is selected afterwards", (await page.getByRole("button", { name: "Whole workspace" }).getAttribute("class")).includes("active"));
check("the open chat session in it was ended", (await page.locator(".sidebar .dot").count()) === 0);

// The right-click menu: Archive on a project, Restore on an archived one.
await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).click({ button: "right" });
const menu = page.getByRole("menu");
await menu.waitFor();
check("right-clicking a project opens its menu", (await menu.getAttribute("aria-label")) === "Actions for demo-chat");
if (SHOTS) await page.screenshot({ path: `${SHOTS}/context-menu.png` });
await page.keyboard.press("Escape");
check("Escape closes the menu", (await menu.count()) === 0);
await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).click({ button: "right" });
await menu.getByRole("menuitem", { name: "Archive" }).click();
await page.waitForFunction(() => document.querySelector(".archived-toggle")?.textContent === "Archived (2)");
check("Archive in the menu archives the project", (await toggle.textContent()) === "Archived (2)");
await toggle.click();
await archivedNav.getByRole("button", { name: "demo-chat" }).click({ button: "right" });
await menu.getByRole("menuitem", { name: "Restore" }).click();
await page.getByRole("button", { name: "Start session in demo-chat" }).waitFor();
check("Restore in the menu restores it and opens it", (await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).getAttribute("class")).includes("active"));

// A taken name is refused with "Already exists".
await page.getByRole("button", { name: "+ New project" }).click();
await page.getByLabel("Project name").fill("Security");
await page.getByRole("button", { name: "Create", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).waitFor();
await toggle.click();
await archivedNav.getByRole("button", { name: "security" }).click({ button: "right" });
await menu.getByRole("menuitem", { name: "Restore" }).click();
await page.getByRole("alert").waitFor();
check("restoring over an existing project is refused", (await page.getByRole("alert").textContent()).includes("Already exists"));

await finish(browser);
