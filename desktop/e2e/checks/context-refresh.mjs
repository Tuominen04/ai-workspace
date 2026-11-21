// The context list follows the conversation: after each reply it shows what the model was
// given with that message, so a grant (or a revoke) made during the session shows up.

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
check("the list does not change before the next message", !(await granted()).includes("router-backups"));
await ask("What is in the backups?");
await replies.nth(1).waitFor();
check("after the next reply the grant is listed under Granted", (await granted()).includes("router-backups"));

// Revoke during the session.
await toTab("Permissions");
await page.getByRole("button", { name: "Revoke /home/me/router-backups" }).click();
await page.getByText("2 permissions").waitFor();
await toTab("Chat");
await ask("And now?");
await replies.nth(2).waitFor();
check("after a revoke the next reply's list leaves it out", !(await granted()).includes("router-backups"));

await finish(browser);
