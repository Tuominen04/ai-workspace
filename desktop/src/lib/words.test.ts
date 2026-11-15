import { describe, expect, it } from "vitest";

import type { Issue } from "./engine";
import { counted, dateText, fileName, fixText, issueWhere } from "./words";

describe("fixText", () => {
  it("words a fix that runs an action", () => {
    expect(
      fixText({ kind: "run", action: "init", purpose: "recreate the standard workspace layout" }),
    ).toBe("Initialize the workspace to recreate the standard workspace layout.");
  });

  it("words a manual step followed by an action", () => {
    expect(fixText({ kind: "fix_then_rerun", step: "fix project.toml", action: "validate" })).toBe(
      "Fix project.toml, then validate again.",
    );
  });

  it("words a manual step on its own", () => {
    expect(fixText({ kind: "manual", step: "recreate 'Specs' under project demo" })).toBe(
      "Recreate 'Specs' under project demo.",
    );
  });

  it("never names a command line", () => {
    const all = [
      fixText({ kind: "run", action: "init", purpose: "x" }),
      fixText({ kind: "fix_then_rerun", step: "y", action: "validate" }),
    ].join(" ");
    expect(all).not.toMatch(/ai-workspace/);
  });
});

describe("issueWhere", () => {
  const base: Issue = {
    severity: "error",
    scope: "workspace",
    subject: "configuration",
    path: "/w/Config/workspace.toml",
    code: "WS-CONFIG-MISSING",
    message: "workspace configuration file is missing",
    fix: { kind: "run", action: "init", purpose: "bootstrap default workspace configuration" },
  };

  it("names the project for project issues", () => {
    expect(issueWhere({ ...base, scope: "project", subject: "demo" })).toBe("Project demo");
  });

  it("names the workspace part otherwise", () => {
    expect(issueWhere(base)).toBe("Workspace (configuration)");
  });
});

describe("small helpers", () => {
  it("counts with the right plural", () => {
    expect(counted(1, "result")).toBe("1 result");
    expect(counted(0, "result")).toBe("0 results");
  });

  it("takes the last part of a path", () => {
    expect(fileName("/home/me/AI/Projects/demo/Knowledge/router.md")).toBe("router.md");
    expect(fileName("router.md")).toBe("router.md");
  });

  it("leaves a non-timestamp as it is", () => {
    expect(dateText("yesterday")).toBe("yesterday");
    expect(dateText("1794690257")).not.toBe("1794690257");
  });
});
