// The sidebar stays in place: a long chat scrolls inside the tab, never the whole window.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 700 } });
await stubEngine(page);
await page.goto(URL);

await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();
for (let n = 0; n < 12; n += 1) {
  const text = `question ${n} ${"with plenty of words ".repeat(20)}`;
  await page.getByLabel("Message").fill(text);
  await page.keyboard.press("Enter");
  await page.locator(".line.ai:not(.streaming)").filter({ hasText: `You asked: question ${n} ` }).waitFor();
}

const pageScrolls = await page.evaluate(
  () => document.scrollingElement.scrollHeight > window.innerHeight + 1,
);
check("the window itself does not scroll", !pageScrolls);
await page.evaluate(() => window.scrollTo(0, 10_000));
const sidebar = await page.locator(".sidebar").boundingBox();
check("the sidebar stays at the top, full height", sidebar.y === 0 && sidebar.height <= 700 + 1, JSON.stringify(sidebar));
const project = await page.locator(".sidebar nav .item", { hasText: "security" }).boundingBox();
check("the projects stay in view", project !== null && project.y >= 0 && project.y < 700, JSON.stringify(project));
const message = await page.getByLabel("Message").boundingBox();
check("the message box stays in view", message.y + message.height <= 700 + 1, JSON.stringify(message));
const lines = await page.locator(".line").last().boundingBox();
check("the latest reply is reachable", lines !== null);

await finish(browser);
