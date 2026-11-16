// A failed reply keeps its question and offers Retry, which sends that same question again.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage();
await stubEngine(page);
await page.goto(URL);
await page.getByRole("button", { name: "security" }).click();
await page.getByRole("tab", { name: "Chat" }).click();
await page.getByRole("button", { name: "Start session in security" }).click();

await page.getByLabel("Message").fill("What about my flaky router?");
await page.keyboard.press("Enter");
await page.locator(".line.note").waitFor();
check("a failed reply says why", (await page.locator(".line.note").textContent()).includes("timed out reading response"));
check("the failed question stays visible", (await page.locator(".line.you").allTextContents()).includes("What about my flaky router?"));

await page.getByRole("button", { name: "Retry" }).click();
await page.locator(".line.ai:not(.streaming)").filter({ hasText: "You asked: What about my flaky router?" }).waitFor();

const sends = await page.evaluate(() => window.__calls.filter((c) => c.command === "chat_send").map((c) => c.args.message));
check("Retry sends the same question again", sends.length === 2 && sends[1] === "What about my flaky router?", JSON.stringify(sends));
check("the failure note is gone after a successful retry", (await page.locator(".line.note").count()) === 0);
check("the question is not duplicated", (await page.locator(".line.you").count()) === 1);

await finish(browser);
