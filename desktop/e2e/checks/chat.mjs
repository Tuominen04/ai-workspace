// The chat: open a session, send, see the reply, end the session.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
// Screenshots are for people, so they are taken at twice the pixel density.
const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

await page.getByRole("tab", { name: "Chat" }).click();
check("chat asks for a project first", (await page.locator(".content").textContent()).includes("Choose a project"));

await page.getByRole("button", { name: "security" }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();
check("the session shows the runtime and its context", (await page.locator(".context summary").textContent()).includes("4 context files"));
await page.locator(".context summary").click();
check("the budget and a cut file are shown", (await page.locator(".context").textContent()).includes("8192-token window") && (await page.locator(".context").textContent()).includes("history.md (11800 of 26000 bytes shown)"));
check("granted files are listed", (await page.locator(".context").textContent()).includes("router-hardening-notes.md"));
check("skipped secret-looking files are listed", (await page.locator(".context").textContent()).includes("wifi-password.md"));
check("the session names its model and keeps the model's instructions", (await page.locator(".agent").textContent()).includes("sec") && (await page.locator(".agent").textContent()).includes("own instructions are kept"));

await page.getByLabel("Message").fill("Is SSH open to the internet?");
await page.keyboard.press("Enter");
await page.locator(".line.ai").filter({ hasText: "You asked" }).waitFor();
const lines = await page.locator(".line").allTextContents();
check("the question and the reply appear in order", lines[0] === "Is SSH open to the internet?" && lines[1]?.startsWith("You asked"), JSON.stringify(lines));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/chat.png` });

await page.getByLabel("Message").fill("half-typed question");
await page.getByRole("tab", { name: "Search" }).click();
await page.getByRole("tab", { name: /^Chat/ }).click();
check("switching tabs keeps the session", await page.getByRole("heading", { name: "Chat · security" }).isVisible());
const kept = await page.locator(".line").allTextContents();
check("switching tabs keeps the conversation", kept[0] === "Is SSH open to the internet?" && kept[1]?.startsWith("You asked"), JSON.stringify(kept));
check("switching tabs keeps a half-typed message", (await page.getByLabel("Message").inputValue()) === "half-typed question");

// A reply that arrives while another tab is open is still there on return.
await page.getByLabel("Message").fill("Second question");
await page.keyboard.press("Enter");
await page.getByRole("tab", { name: "Overview" }).click();
await page.waitForTimeout(400);
await page.getByRole("tab", { name: /^Chat/ }).click();
check("a reply that arrived on another tab is shown", (await page.locator(".line.ai").allTextContents()).some((t) => t.includes("Second question")));

await page.getByRole("button", { name: "End session" }).click();
// The conversation had answers, so ending offers a note first (ADR-0012).
await page.getByRole("button", { name: "End without a note" }).click();
await page.getByText("1 temporary permission revoked").waitFor();
check("ending the session reports the revoked permissions", true);

await finish(browser);
