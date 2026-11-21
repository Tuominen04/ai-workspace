// The context list follows the session: a grant or revoke shows at once, a sent message shows
// what it gets before the reply comes, and each reply shows what the model was given.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();
const page = await browser.newPage({ viewport: { width: 1100, height: 750 } });
await stubEngine(page);
await page.goto(URL);

const replies = page.locator(".line.ai:not(.streaming):not(.thinking)");
const ask = async (text) => {
  await page.getByLabel("Message").fill(text);
  await page.waitForFunction(() => !document.querySelector(".composer button[type=submit]").disabled);
  await page.locator(".composer button[type=submit]").click();
};
const granted = async () => {
  const group = page.locator(".context-group", { has: page.locator(".label", { hasText: /^Granted$/ }) });
  return (await group.count()) === 0 ? "" : await group.textContent();
};
const toTab = async (name) => page.getByRole("tab", { name }).click();

await page.getByRole("button", { name: "security", exact: true }).click();
await page.getByRole("button", { name: "Start session in security" }).click();
await ask("First question");
await replies.nth(0).waitFor();
await page.locator(".chat details.context summary").click();
check("before a grant, nothing is listed under Granted", !(await granted()).includes("router-backups"));

// Grant during the session.
await toTab("Permissions");
await page.getByLabel("Path to grant").fill("/home/me/router-backups");
await page.getByRole("button", { name: "Grant" }).click();
await page.getByText("3 permissions").waitFor();
await toTab("Chat");
await page.waitForFunction(() => document.querySelector(".chat details.context")?.textContent.includes("router-backups"));
check("a grant shows under Granted at once", (await granted()).includes("router-backups"));
await ask("What is in the backups? Answer slowly");
await page.locator(".line.ai.streaming").waitFor();
check("while the reply is written the list shows what the message got", (await granted()).includes("router-backups"));
await replies.nth(1).waitFor();
check("after the reply the grant is still listed", (await granted()).includes("router-backups"));

// Revoke during the session.
await toTab("Permissions");
await page.getByRole("button", { name: "Revoke /home/me/router-backups" }).click();
await page.getByText("2 permissions").waitFor();
await toTab("Chat");
await page.waitForFunction(() => !document.querySelector(".chat details.context")?.textContent.includes("router-backups"));
check("a revoke drops it from the list at once", !(await granted()).includes("router-backups"));
await ask("And now?");
await replies.nth(2).waitFor();
check("and the next reply's list leaves it out too", !(await granted()).includes("router-backups"));

const refreshes = await page.evaluate(() => window.__calls.filter((c) => c.command === "chat_context").map((c) => c.args.message ?? null));
check(
  "the context is refreshed for each grant, revoke and sent message",
  JSON.stringify(refreshes) === JSON.stringify(["First question", null, "What is in the backups? Answer slowly", null, "And now?"]),
  JSON.stringify(refreshes),
);

await finish(browser);
