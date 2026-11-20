// A chat longer than the model's window: the chat says, once each time it grows, that the
// model only sees the latest messages, and replies keep coming.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage();
await stubEngine(page);
await page.goto(URL);

await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await page.getByRole("heading", { name: "Chat · security" }).waitFor();

const ask = async (text) => {
  await page.getByLabel("Message").fill(text);
  await page.keyboard.press("Enter");
  await page.locator(".line.ai:not(.streaming)").filter({ hasText: `You asked: ${text}` }).waitFor();
};
const notes = () => page.locator(".line.note").filter({ hasText: "longer than the model's window" });

await ask("first question");
check("no note while everything fits", (await notes().count()) === 0);
await ask("a long chat question");
check("the chat says the model now sees only the latest messages", (await notes().count()) === 1 && (await notes().first().textContent()).includes("2 exchanges"));
await ask("same length again");
check("the note is not repeated while nothing more is left out", (await notes().count()) === 1);
await ask("long chat once more");
check("it says so again when more is left out", (await notes().count()) === 2);

await finish(browser);
