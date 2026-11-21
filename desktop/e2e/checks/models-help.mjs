// When Ollama lists no models, a "?" by the model hint explains where each Ollama keeps its
// models; with models listed there is nothing to explain.

import { check, finish, launch, SHOTS, stubEngine, URL } from "../support.mjs";

const browser = await launch();

async function openModelEditor(page) {
  await page.goto(URL);
  await page.getByRole("button", { name: "security", exact: true }).click();
  await page.getByRole("tab", { name: "Overview" }).click();
  await page.getByRole("heading", { name: "Security" }).waitFor();
  await page.getByRole("button", { name: "Change" }).click();
}

{
  // An Ollama with no models (or none answering): the list comes back empty.
  const page = await browser.newPage({ viewport: { width: 1100, height: 750 }, deviceScaleFactor: SHOTS ? 2 : 1 });
  await stubEngine(page, { failures: { models_installed: { kind: "Bridge", message: "nothing listed" } } });
  await openModelEditor(page);
  await page.getByText("Ollama did not list any models").waitFor();

  const help = page.getByRole("button", { name: "Where are my models?" });
  check("a ? sits by the hint when no models are listed", await help.isVisible());
  const note = page.getByRole("note", { name: "Where your models are" });
  check("the explanation is hidden until asked for", (await note.count()) === 0);

  await help.click();
  const text = await note.textContent();
  check(
    "the ? names each Ollama's model folder",
    text.includes("/usr/share/ollama/.ollama/models") && text.includes("~/.ollama/models") && text.includes("ai-workspace-ollama"),
  );
  check("and what to do", text.includes("sudo systemctl start ollama") && text.includes("ollama pull"));
  check("the ? says it is open", (await help.getAttribute("aria-expanded")) === "true");
  if (SHOTS) await page.screenshot({ path: `${SHOTS}/models-help.png` });

  await help.click();
  check("a second click hides it again", (await note.count()) === 0);
  await page.close();
}

{
  // Models listed: no ?.
  const page = await browser.newPage();
  await stubEngine(page);
  await openModelEditor(page);
  await page.getByText("Installed in Ollama: qwen3.6:latest, sec:latest.").waitFor();
  check("no ? when models are listed", (await page.getByRole("button", { name: "Where are my models?" }).count()) === 0);
  await page.close();
}

await finish(browser);
