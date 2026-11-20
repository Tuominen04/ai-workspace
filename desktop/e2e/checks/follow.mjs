// The conversation follows a reply as it is written only while the person is at its end.
// Scrolling up to read stops it, also when the reply finishes; scrolling back down to the end
// starts it again; sending a message follows again.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 700 } });
await stubEngine(page);
await page.goto(URL);

const words = "with plenty of words to read through ".repeat(25);
const replies = page.locator(".line.ai:not(.streaming)");
const send = async (text) => {
  await page.getByLabel("Message").fill(text);
  await page.waitForFunction(() => !document.querySelector(".composer button[type=submit]").disabled);
  await page.locator(".composer button[type=submit]").click();
};
const position = () =>
  page.locator(".transcript").evaluate((box) => ({
    top: box.scrollTop,
    fromEnd: box.scrollHeight - box.scrollTop - box.clientHeight,
  }));
const atEnd = async () => (await position()).fromEnd <= 24;
// A person scrolling with the mouse wheel over the conversation.
const wheel = async (deltaY) => {
  const box = await page.locator(".transcript").boundingBox();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.wheel(0, deltaY);
  await page.waitForTimeout(100);
};

await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
for (let n = 1; n <= 4; n++) {
  await send(`question ${n} ${words}`);
  await replies.nth(n - 1).waitFor();
}
check("the conversation follows replies while at its end", await atEnd());

// Scrolling up while a long reply is written: the view stays where the person reads.
await send(`answer slowly please ${words}`);
await page.locator(".line.ai.streaming").waitFor();
await wheel(-600);
const reading = await position();
check("scrolling up stops following", reading.fromEnd > 24, JSON.stringify(reading));
await replies.nth(4).waitFor();
const after = await position();
check(
  "the finished reply does not jump the view to the end",
  Math.abs(after.top - reading.top) <= 2 && after.fromEnd > 24,
  JSON.stringify({ reading, after }),
);

// Scrolling back down to the end during a reply: following starts again.
await wheel(100_000);
await send(`and again slowly ${words}`);
await page.locator(".line.ai.streaming").waitFor();
await wheel(-600);
await wheel(100_000);
await replies.nth(5).waitFor();
check("scrolling back down to the end follows the reply again", await atEnd(), JSON.stringify(await position()));

// Sending a message while scrolled up shows it and follows the reply.
await wheel(-600);
await send("one more slowly");
await replies.nth(6).waitFor();
check("sending a message follows again", await atEnd(), JSON.stringify(await position()));

await finish(browser);
