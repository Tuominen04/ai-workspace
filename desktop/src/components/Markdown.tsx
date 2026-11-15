import ReactMarkdown, { type Components } from "react-markdown";
import remarkGfm from "remark-gfm";

/**
 * A model reply, rendered as Markdown: headings, lists, emphasis, code and tables.
 *
 * Raw HTML in the reply is never rendered (react-markdown's default). Links and images are
 * shown as text instead of being followed or loaded: clicking a link would navigate the app's
 * own window away, and an image would be a request to somewhere else. Everything stays on
 * this computer.
 */
const components: Components = {
  a: ({ children, href }) => (
    <span className="md-link" title={href}>
      {children}
      {href && typeof children === "string" && children !== href ? ` (${href})` : null}
    </span>
  ),
  img: ({ alt, src }) => (
    <span className="md-image">[image: {alt || (typeof src === "string" ? src : "")}]</span>
  ),
};

export function Markdown({ text }: { text: string }) {
  return (
    <div className="markdown">
      <ReactMarkdown remarkPlugins={[remarkGfm]} components={components}>
        {text}
      </ReactMarkdown>
    </div>
  );
}
