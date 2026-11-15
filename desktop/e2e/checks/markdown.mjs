// Model replies are rendered as Markdown, and the chat sits centred in a wide window.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 900 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);
await page.getByRole("button", { name: "security" }).click();
await page.getByRole("tab", { name: "Chat" }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByLabel("Message").fill("Testing security agent. What are your rules?");
await page.keyboard.press("Enter");
await page.locator(".line.ai h2").waitFor();

const reply = page.locator(".line.ai").last();
check("headings are rendered", (await reply.locator("h2").textContent()) === "My Rules (short version)");
check("numbered and bulleted lists are rendered", (await reply.locator("ol > li").count()) === 3 && (await reply.locator("ul > li").count()) === 2);
check("bold and inline code are rendered", (await reply.locator("strong").first().textContent()) === "Exact Fedora commands" && (await reply.locator("code").first().textContent()) === "dnf");
check("the question stays plain text", (await page.locator(".line.you").last().locator("h2, strong, code").count()) === 0);

const panel = await page.locator(".chat").boundingBox();
const content = await page.locator(".content").boundingBox();
const left = panel.x - content.x;
const right = content.x + content.width - (panel.x + panel.width);
check("the chat is centred in a wide window", Math.abs(left - right) < 30, `left ${left}, right ${right}`);
if (SHOTS) await page.screenshot({ path: `${SHOTS}/markdown.png` });

await finish(browser);
