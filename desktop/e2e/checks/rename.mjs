// Renaming projects (ADR-0015 point 7): from the Overview and the right-click menu, the folder
// preview, a session ended first, refusals, and no Rename for archived projects.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

const projectsNav = page.getByRole("navigation", { name: "Projects", exact: true });
const dialog = page.getByRole("dialog");
const input = page.getByLabel("New name");

// From the Overview, with a chat session open.
await projectsNav.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("button", { name: "Rename…" }).click();
await dialog.waitFor();
check("the dialog starts from the current name", (await input.inputValue()) === "Security");
check("an unchanged name cannot be saved", await page.getByRole("button", { name: "Rename", exact: true }).isDisabled());
await input.fill("Home Security");
check("it shows the folder the name becomes", (await dialog.textContent()).includes("Projects/home-security"));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/rename.png` });
await page.getByRole("button", { name: "Rename", exact: true }).click();
await page.getByRole("heading", { name: "Home Security" }).waitFor();
check("the project is listed under its new folder and selected", (await projectsNav.getByRole("button", { name: "home-security" }).getAttribute("class")).includes("active"));
check("the old name is gone", (await projectsNav.getByRole("button", { name: "security", exact: true }).count()) === 0);
check("the open chat session in it was ended", (await page.locator(".sidebar .dot").count()) === 0);

// From the menu: a taken folder and a reserved name are refused; Escape cancels.
await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).click({ button: "right" });
await page.getByRole("menuitem", { name: "Rename…" }).click();
await input.waitFor();
await input.fill("home security");
await page.getByRole("button", { name: "Rename", exact: true }).click();
await page.getByRole("alert").waitFor();
check("a taken folder is refused", (await page.getByRole("alert").textContent()).includes("Already exists"));
await input.fill("Archive");
await page.getByRole("button", { name: "Rename", exact: true }).click();
await page.getByRole("alert").filter({ hasText: "invalid project name" }).waitFor();
check("a reserved name is refused", true);
await input.fill("lab/ok");
check("characters the folder cannot hold are flagged before saving", (await dialog.textContent()).includes("Letters, digits"));
await page.keyboard.press("Escape");
check("Escape cancels", (await dialog.count()) === 0 && (await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).count()) === 1);

// Archived projects are not renamed.
await page.getByRole("button", { name: /^Archived \(\d+\)$/ }).click();
await page.getByRole("navigation", { name: "Archived projects" }).getByRole("button", { name: "old-lab" }).click({ button: "right" });
const items = (await page.getByRole("menuitem").allTextContents()).map((t) => t.trim());
check("an archived project's menu has no Rename", !items.includes("Rename…"), JSON.stringify(items));

await finish(browser);
