import { messageOf } from "./engine";

/** A failed call, as panels keep it: the message, and the kind when the shell gave one. */
export type FailureInfo = { message: string; kind: string | null };

export function failureOf(thrown: unknown): FailureInfo {
  return { message: messageOf(thrown), kind: (thrown as { kind?: string }).kind ?? null };
}
