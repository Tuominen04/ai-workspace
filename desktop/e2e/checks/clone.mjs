// Cloning a project (ADR-0015): from the Overview and from the right-click menu, the clone
// opens on its chat, and a taken name is refused.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

const projectsNav = page.getByRole("navigation", { name: "Projects", exact: true });

// From the Overview.
await projectsNav.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("heading", { name: "Security" }).waitFor();
await page.getByRole("button", { name: "Clone" }).click();
await page.getByRole("button", { name: "Start session in security-clone" }).waitFor();
check("the clone is listed", (await projectsNav.textContent()).includes("security-clone"));
check("and selected, on its chat", (await projectsNav.getByRole("button", { name: "security-clone" }).getAttribute("class")).includes("active") && (await page.getByRole("tab", { name: "Chat" }).getAttribute("aria-selected")) === "true");
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("heading", { name: "Security (Clone)" }).waitFor();
check("its name says it is a clone", true);

// From the right-click menu, with Clone first.
await projectsNav.getByRole("button", { name: "demo-chat", exact: true }).click({ button: "right" });
const items = (await page.getByRole("menuitem").allTextContents()).map((t) => t.trim());
check("a project's menu offers Clone and Archive", JSON.stringify(items) === JSON.stringify(["Clone", "Archive"]), JSON.stringify(items));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/clone-menu.png` });
await page.getByRole("menuitem", { name: "Clone" }).click();
await page.getByRole("button", { name: "Start session in demo-chat-clone" }).waitFor();
check("Clone in the menu clones the project", (await projectsNav.textContent()).includes("demo-chat-clone"));

// A taken name is refused.
await projectsNav.getByRole("button", { name: "security", exact: true }).click({ button: "right" });
await page.getByRole("menuitem", { name: "Clone" }).click();
await page.getByRole("alert").waitFor();
check("cloning when the clone exists is refused", (await page.getByRole("alert").textContent()).includes("Already exists"));
await page.getByRole("tab", { name: "Search" }).click();
check("clicking elsewhere dismisses the message", (await page.getByRole("alert").count()) === 0);

// The same refusal on the Overview: a click on it dismisses it, and so does another project.
await projectsNav.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("heading", { name: "Security", exact: true }).waitFor();
await page.getByRole("button", { name: "Clone", exact: true }).click();
await page.getByRole("alert").waitFor();
await page.getByRole("alert").click();
check("clicking the message dismisses it", (await page.getByRole("alert").count()) === 0);
await page.getByRole("button", { name: "Clone", exact: true }).click();
await page.getByRole("alert").waitFor();
await page.keyboard.press("Escape");
check("Escape dismisses it", (await page.getByRole("alert").count()) === 0);
await page.getByRole("button", { name: "Clone", exact: true }).click();
await page.getByRole("alert").waitFor();
await projectsNav.getByRole("button", { name: "security-clone", exact: true }).click();
await page.getByRole("tab", { name: "Overview" }).click();
await page.getByRole("heading", { name: "Security (Clone)" }).waitFor();
check("another project never shows the old message", (await page.getByRole("alert").count()) === 0);

await finish(browser);
