// The theme switch: follows the system, a fixed choice applies at once and survives a reload.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();
// The system prefers light here, so a dark result can only come from the switch.
const page = await browser.newPage({
  colorScheme: "light",
  viewport: { width: 1100, height: 750 },
  deviceScaleFactor: SHOTS ? 2 : 1,
});
await stubEngine(page);
await page.goto(URL);
await page.getByRole("button", { name: "security" }).click();
await page.getByRole("heading", { name: "Security" }).waitFor();

const background = () => page.evaluate(() => getComputedStyle(document.body).backgroundColor);
const light = await background();
check("the theme follows the system at first", (await page.getByLabel("Theme").inputValue()) === "system");

await page.getByLabel("Theme").selectOption("dark");
const dark = await background();
check("choosing Dark changes the colours at once", dark !== light, `${light} → ${dark}`);
if (SHOTS) await page.screenshot({ path: `${SHOTS}/overview-dark.png` });

await page.reload();
await page.getByLabel("Theme").waitFor();
check("the choice survives a reload", (await page.getByLabel("Theme").inputValue()) === "dark" && (await background()) === dark);

await page.getByLabel("Theme").selectOption("system");
check("System goes back to the system's colours", (await background()) === light);

await finish(browser);
