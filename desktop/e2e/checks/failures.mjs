// What a person sees when something fails: a refused gate, a runtime that does not answer.

import { check, finish, launch, stubEngine, URL } from "../support.mjs";

const browser = await launch();

{
  const page = await browser.newPage();
  await stubEngine(page, {
    failures: {
      project_show: {
        kind: "ValidationGateBlocked",
        message: "cannot run project show because workspace validation has blocking issues",
      },
    },
  });
  await page.goto(URL);
  await page.getByRole("button", { name: "security" }).click();
  await page.getByRole("alert").waitFor();
  const text = await page.getByRole("alert").textContent();
  check("a refused gate says so and points to Validation", text.includes("blocking issues") && text.includes("Validation tab"));
  await page.close();
}

{
  const page = await browser.newPage();
  await stubEngine(page, {
    failures: { chat_send: { kind: "RuntimeRequestFailed", message: "request to the AI runtime failed: connection refused" } },
  });
  await page.goto(URL);
  await page.getByRole("button", { name: "security" }).click();
  await page.getByRole("tab", { name: "Chat" }).click();
  await page.getByRole("button", { name: "Start session in security" }).click();
  await page.getByLabel("Message").fill("hello");
  await page.getByRole("button", { name: "Send" }).click();
  await page.locator(".line.note").waitFor();
  check("a failed reply is shown and the chat stays usable", (await page.locator(".line.note").textContent()).includes("connection refused") && (await page.getByLabel("Message").isEnabled()));
  await page.close();
}

{
  const page = await browser.newPage();
  await page.goto(URL);
  check("in a plain browser the page explains itself", (await page.locator("main").textContent()).includes("npm run tauri dev"));
  await page.close();
}

await finish(browser);
