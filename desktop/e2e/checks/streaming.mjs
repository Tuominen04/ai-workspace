// A reply is shown as it is written (ADR-0013), then replaced by the finished reply.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage();
await stubEngine(page);
await page.goto(URL);
await page.getByRole("button", { name: "security" }).click();
await page.getByRole("tab", { name: "Chat" }).click();
await page.getByRole("button", { name: "Start session in security" }).click();

await page.getByLabel("Message").fill("Please answer slowly");
await page.keyboard.press("Enter");
await page.locator(".line.ai.streaming").waitFor();
const partial = await page.locator(".line.ai.streaming").textContent();
check("part of the reply shows while it is being written", partial.length > 0 && !partial.includes("Please answer slowly"), JSON.stringify(partial));
check("Thinking… is replaced by the growing text", (await page.locator(".thinking").count()) === 0);

await page.locator(".line.ai:not(.streaming)").filter({ hasText: "You asked: Please answer slowly" }).waitFor();
check("the finished reply replaces the streaming one", (await page.locator(".line.ai.streaming").count()) === 0);
check("the composer is usable again when the reply is done", await page.getByLabel("Message").isEnabled());

await finish(browser);
