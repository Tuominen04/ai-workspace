// Closing the app with an answered chat open: the shell holds the close and sends the chat's
// project; the chat asks whether to save a note, and the window closes only after an answer.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage();
await stubEngine(page);
await page.goto(URL);

const calls = (command) => page.evaluate((c) => window.__calls.filter((x) => x.command === c).length, command);
const quit = () => page.evaluate(() => window.__quit === true);
const asking = page.getByRole("group", { name: "End session" });

await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByLabel("Message").fill("What is open?");
await page.keyboard.press("Enter");
await page.locator(".line.ai:not(.streaming)").first().waitFor();

// Somewhere else in the app when the window is closed.
await page.getByRole("button", { name: "Whole workspace" }).click();
await page.evaluate(() => window.__emit("close-requested", "security"));
await asking.waitFor();
check("closing shows the chat and asks about a note", (await asking.textContent()).includes("AI Workspace is closing") && (await page.getByRole("heading", { name: "Chat · security" }).isVisible()));
check("nothing is closed yet", !(await quit()));

await page.getByRole("button", { name: "Keep chatting" }).click();
check("Keep chatting calls the close off", (await asking.count()) === 0 && !(await quit()));

// Save a note, then the window closes.
await page.evaluate(() => window.__emit("close-requested", "security"));
await page.getByRole("button", { name: "Draft a note" }).click();
await page.getByRole("button", { name: "Save note and close" }).click();
await page.waitForFunction(() => window.__quit === true);
check("saving the note closes the session, then the window", (await calls("note_save")) === 1 && (await calls("chat_close")) === 1 && (await quit()));

await page.close();

// Or close without a note.
const second = await browser.newPage();
await stubEngine(second);
await second.goto(URL);
await second.getByRole("button", { name: "security", exact: true }).click();
await second.getByRole("button", { name: "Start session in security" }).click();
await second.getByLabel("Message").fill("Anything?");
await second.keyboard.press("Enter");
await second.locator(".line.ai:not(.streaming)").first().waitFor();
await second.evaluate(() => window.__emit("close-requested", "security"));
await second.getByRole("button", { name: "Close without a note" }).click();
await second.waitForFunction(() => window.__quit === true);
const saved = await second.evaluate(() => window.__calls.some((c) => c.command === "note_save"));
check("closing without a note saves nothing and closes", !saved);

await second.close();

// Closed while a reply is still being written: asked at once, answerable when the reply is done.
const third = await browser.newPage();
await stubEngine(third);
await third.goto(URL);
const done = third.locator(".line.ai:not(.streaming):not(.thinking)");
const ask = async (text) => {
  await third.getByLabel("Message").fill(text);
  await third.waitForFunction(() => !document.querySelector(".composer button[type=submit]").disabled);
  await third.locator(".composer button[type=submit]").click();
};
await third.getByRole("button", { name: "security", exact: true }).click();
await third.getByRole("button", { name: "Start session in security" }).click();
await ask("First question");
await done.first().waitFor();
await ask("Now answer slowly");
await third.locator(".line.ai.streaming").waitFor();
await third.evaluate(() => window.__emit("close-requested", "security"));
const closeNow = third.getByRole("button", { name: "Close without a note" });
await closeNow.waitFor();
check("closing mid-reply asks at once, but waits for the reply to answer", await closeNow.isDisabled());
await done.filter({ hasText: "You asked: Now answer slowly" }).waitFor();
await closeNow.click();
await third.waitForFunction(() => window.__quit === true);
check("once the reply is done the question can be answered", await third.evaluate(() => window.__quit === true));

await finish(browser);
