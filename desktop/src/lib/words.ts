// How the interface words what the core returns as data. Pure, so it is tested directly.

import type { Fix, FixAction, Issue } from "./engine";

/** The workspace action a fix points to, as a person would say it. */
export function actionLabel(action: FixAction): string {
  switch (action) {
    case "init":
      return "Initialize the workspace";
    case "validate":
      return "Validate again";
  }
}

/** One line telling the person how to clear an issue. */
export function fixText(fix: Fix): string {
  switch (fix.kind) {
    case "run":
      return `${actionLabel(fix.action)} to ${fix.purpose}.`;
    case "fix_then_rerun":
      return `${capitalize(fix.step)}, then ${actionLabel(fix.action).toLowerCase()}.`;
    case "manual":
      return `${capitalize(fix.step)}.`;
  }
}

/** Where an issue belongs: the workspace, or a named project. */
export function issueWhere(issue: Issue): string {
  return issue.scope === "project" ? `Project ${issue.subject}` : `Workspace (${issue.subject})`;
}

/** "1 result", "3 results". */
export function counted(n: number, noun: string): string {
  return `${n} ${noun}${n === 1 ? "" : "s"}`;
}

/** The last part of a path, for compact lists; the full path goes in a tooltip. */
export function fileName(path: string): string {
  const parts = path.split(/[\\/]/).filter((part) => part.length > 0);
  return parts[parts.length - 1] ?? path;
}

/** A Unix-seconds timestamp as a local date, or the text as given if it is not one. */
export function dateText(unixSeconds: string): string {
  const seconds = Number(unixSeconds);
  if (!Number.isFinite(seconds) || seconds <= 0) return unixSeconds;
  return new Date(seconds * 1000).toLocaleString();
}

function capitalize(text: string): string {
  return text.length === 0 ? text : text[0]!.toUpperCase() + text.slice(1);
}
