// In a long chat the end and close questions appear at the top, next to End session, in view
// without scrolling; writing another message instead of answering means keep chatting.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 700 }, deviceScaleFactor: SHOTS ? 2 : 1 });
await stubEngine(page);
await page.goto(URL);

const quit = () => page.evaluate(() => window.__quit === true);
const asking = page.getByRole("group", { name: "End session" });
const inView = (locator) =>
  locator.evaluate((element) => {
    const box = element.getBoundingClientRect();
    return box.top >= 0 && box.bottom <= window.innerHeight;
  });
// Sends a message once the previous reply is done (Send is enabled again).
async function say(text) {
  await page.getByLabel("Message").fill(text);
  await page.waitForFunction(() => !document.querySelector(".composer button[type=submit]").disabled);
  await page.locator(".composer button[type=submit]").click();
}
const above = (first, second) =>
  page.evaluate(
    ([a, b]) => document.querySelector(a).getBoundingClientRect().top < document.querySelector(b).getBoundingClientRect().top,
    [first, second],
  );

await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
for (let n = 1; n <= 8; n++) {
  await say(`Question ${n}: what else should I harden on the router?`);
  await page.locator(".line.ai:not(.streaming)").nth(n - 1).waitFor();
}

// End session, pressed at the top of a long chat.
await page.getByRole("button", { name: "End session" }).click();
await asking.waitFor();
check("ending asks at the top, above the conversation", await above(".chat .ending", ".chat .transcript"));
check("the question is in view without scrolling", await inView(asking));
if (SHOTS) await page.screenshot({ path: `${SHOTS}/prompt-top.png` });

// Writing on instead of answering keeps the session.
check("the message box stays usable while asking", await page.getByLabel("Message").isEditable());
await say("One more thing: and the guest network?");
await page.locator(".line.ai:not(.streaming)").nth(8).waitFor();
check("sending a message instead calls the ending off", (await asking.count()) === 0);
check("and the session stays open", !(await page.evaluate(() => window.__calls.some((c) => c.command === "chat_close"))));

// Closing the window from another tab: the chat opens with the question in view.
await page.getByRole("button", { name: "Whole workspace" }).click();
await page.evaluate(() => window.__emit("close-requested", "security"));
await asking.waitFor();
check("closing shows the question at the top, in view", (await inView(asking)) && (await above(".chat .ending", ".chat .transcript")));

await say("Wait, one last question.");
await page.locator(".line.ai:not(.streaming)").nth(9).waitFor();
check("sending a message instead calls the close off", (await asking.count()) === 0 && !(await quit()));

await finish(browser);
