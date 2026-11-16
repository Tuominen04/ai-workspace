// @vitest-environment jsdom
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { Markdown } from "./Markdown";

const html = (text: string) => renderToStaticMarkup(<Markdown text={text} />);

describe("Markdown replies", () => {
  it("renders headings, lists, emphasis and code", () => {
    const out = html("## My Rules\n\n1. **Exact commands** like `dnf`\n2. *Be direct*\n");
    expect(out).toContain("<h2>My Rules</h2>");
    expect(out).toContain("<ol>");
    expect(out).toContain("<strong>Exact commands</strong>");
    expect(out).toContain("<code>dnf</code>");
    expect(out).toContain("<em>Be direct</em>");
  });

  it("renders tables and fenced code", () => {
    const out = html("| Port | State |\n| --- | --- |\n| 22 | open |\n\n```\nfirewall-cmd --list-all\n```\n");
    expect(out).toContain("<table>");
    expect(out).toContain("<td>22</td>");
    expect(out).toContain("<pre><code>firewall-cmd --list-all");
  });

  it("never renders raw HTML from a reply", () => {
    const out = html('Hello <script>alert(1)</script> <img src="http://example.com/x.png">');
    expect(out).not.toContain("<script");
    expect(out).not.toContain("<img");
  });

  it("shows links and images as text instead of following or loading them", () => {
    const out = html("See [Fedora docs](https://docs.fedoraproject.org) and ![diagram](http://example.com/d.png)");
    expect(out).not.toContain("<a ");
    expect(out).not.toContain("<img");
    expect(out).toContain("Fedora docs (https://docs.fedoraproject.org)");
    expect(out).toContain("[image: diagram]");
  });
});
