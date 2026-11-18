// Deleting projects (ADR-0015): a confirmation first, the Trash by default, a second warning
// before erasing when the Trash cannot take the folder, and Cancel deleting nothing.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

const projectsNav = page.getByRole("navigation", { name: "Projects", exact: true });
const dialog = page.getByRole("alertdialog");
const deleteCalls = () => page.evaluate(() => window.__calls.filter((c) => c.command === "project_delete"));

// Cancel deletes nothing.
await projectsNav.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("button", { name: "Delete…" }).click();
await dialog.waitFor();
check("Delete asks first, naming the project and the Trash", (await dialog.textContent()).includes("Delete security?") && (await dialog.textContent()).includes("Trash"));
check("the safe choice has the focus", await page.getByRole("button", { name: "Cancel" }).evaluate((el) => el === document.activeElement));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/delete-confirm.png` });
await page.keyboard.press("Escape");
check("Escape cancels and nothing is deleted", (await dialog.count()) === 0 && (await deleteCalls()).length === 0);

// Move to Trash, with a chat session open in the project.
await page.getByRole("tab", { name: "Chat" }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("button", { name: "Delete…" }).click();
await page.getByRole("button", { name: "Move to Trash" }).click();
await page.getByRole("status").filter({ hasText: "Moved security to the Trash" }).waitFor();
const [first] = await deleteCalls();
check("the first try never erases", first?.args?.erase === false, JSON.stringify(first));
check("the project is gone and the whole workspace is shown", (await projectsNav.getByRole("button", { name: "security", exact: true }).count()) === 0 && (await page.getByRole("button", { name: "Whole workspace" }).getAttribute("class")).includes("active"));
check("the chat session in it was ended", (await page.locator(".sidebar .dot").count()) === 0);

// The Trash cannot take it: a second warning before erasing.
await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).click({ button: "right" });
await page.getByRole("menuitem", { name: "Delete…" }).click();
await page.getByRole("button", { name: "Move to Trash" }).click();
await page.getByRole("heading", { name: "Erase demo-chat permanently?" }).waitFor();
check("the second warning says why and that it cannot be undone", (await dialog.textContent()).includes("no trash folder") && (await dialog.textContent()).includes("cannot be restored"));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/delete-erase.png` });
await page.getByRole("button", { name: "Cancel" }).click();
check("Cancel at the second warning keeps the project", (await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).count()) === 1);
await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).click({ button: "right" });
await page.getByRole("menuitem", { name: "Delete…" }).click();
await page.getByRole("button", { name: "Move to Trash" }).click();
await page.getByRole("button", { name: "Erase permanently" }).click();
await page.getByRole("status").filter({ hasText: "Erased demo-chat" }).waitFor();
const calls = await deleteCalls();
check("erasing is sent only after the second confirmation", calls.at(-1)?.args?.erase === true && calls.filter((c) => c.args.erase).length === 1);

// An archived project can be deleted too.
await page.getByRole("button", { name: /^Archived \(\d+\)$/ }).click();
const archivedNav = page.getByRole("navigation", { name: "Archived projects" });
await archivedNav.getByRole("button", { name: "old-lab" }).click();
await page.getByRole("button", { name: "Delete…" }).click();
await page.getByRole("button", { name: "Move to Trash" }).click();
await page.getByRole("status").filter({ hasText: "Moved old-lab to the Trash" }).waitFor();
check("an archived project is deleted from the archive", (await deleteCalls()).at(-1)?.args?.archived === true);

await finish(browser);
