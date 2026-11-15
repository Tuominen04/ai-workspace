// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";

import { applyTheme, isTheme, savedTheme } from "./theme";

describe("theme", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("follows the system until something is chosen", () => {
    expect(savedTheme()).toBe("system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
  });

  it("applies and remembers a fixed choice", () => {
    applyTheme("dark");
    expect(document.documentElement.getAttribute("data-theme")).toBe("dark");
    expect(savedTheme()).toBe("dark");
  });

  it("going back to system forgets the choice", () => {
    applyTheme("light");
    applyTheme("system");
    expect(document.documentElement.hasAttribute("data-theme")).toBe(false);
    expect(savedTheme()).toBe("system");
  });

  it("ignores a stored value it does not know", () => {
    window.localStorage.setItem("ai-workspace.theme", "purple");
    expect(savedTheme()).toBe("system");
    expect(isTheme("purple")).toBe(false);
  });
});
