// Ending a session offers a note: drafted by the model, edited, saved only on request.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();

async function openAndAsk(page) {
  await page.goto(URL);
  await page.getByRole("button", { name: "security" }).click();
  await page.getByRole("tab", { name: "Chat" }).click();
  await page.getByRole("button", { name: "Start session in security" }).click();
  await page.getByLabel("Message").fill("Is SSH open?");
  await page.keyboard.press("Enter");
  await page.locator(".line.ai").filter({ hasText: "You asked" }).waitFor();
}

{
  const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
  await stubEngine(page);
  await openAndAsk(page);

  await page.getByRole("button", { name: "End session" }).click();
  await page.getByRole("group", { name: "End session" }).waitFor();
  check("ending offers a note instead of closing at once", !(await page.evaluate(() => window.__calls.some((c) => c.command === "chat_close"))));

  await page.getByRole("button", { name: "Draft a note" }).click();
  await page.getByRole("textbox", { name: "Session note" }).waitFor();
  check("the model's draft is shown for editing", (await page.getByRole("textbox", { name: "Session note" }).inputValue()).includes("Router hardening"));
  await page.getByRole("textbox", { name: "Session note" }).fill("# Router hardening\n- Closed WAN SSH\n- Next: guest VLAN");
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/note.png` });
  await page.getByRole("button", { name: "Save note and end" }).click();
  await page.getByText("Note saved to").waitFor();

  const calls = await page.evaluate(() => window.__calls);
  const save = calls.find((c) => c.command === "note_save");
  check("the edited text is what gets saved", save?.args?.text.includes("Closed WAN SSH") && save?.args?.project === "security", JSON.stringify(save));
  check("saving happens before the session closes", calls.findIndex((c) => c.command === "note_save") < calls.findIndex((c) => c.command === "chat_close"));
  await page.close();
}

{
  const page = await browser.newPage();
  await stubEngine(page);
  await openAndAsk(page);
  await page.getByRole("button", { name: "End session" }).click();
  await page.getByRole("button", { name: "End without a note" }).click();
  await page.getByText("1 temporary permission revoked").waitFor();
  check("ending without a note writes nothing", !(await page.evaluate(() => window.__calls.some((c) => c.command === "note_save"))));
  await page.close();
}

{
  const page = await browser.newPage();
  await stubEngine(page);
  await openAndAsk(page);
  await page.getByRole("button", { name: "End session" }).click();
  await page.getByRole("button", { name: "Keep chatting" }).click();
  check("Keep chatting goes back to the conversation", await page.getByLabel("Message").isVisible());
  await page.close();
}

await finish(browser);
