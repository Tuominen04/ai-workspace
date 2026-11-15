import { useCallback, useEffect, useState } from "react";

import { messageOf } from "./engine";

/** What an engine call has produced so far. */
export type Loaded<T> =
  | { state: "loading" }
  | { state: "ready"; value: T }
  | { state: "failed"; message: string; kind: string | null };

/**
 * Runs `load` now and whenever `key` changes, and again on `reload()`. An answer that arrives
 * after a newer request started is dropped, so a slow reply never overwrites a fresh one.
 */
export function useLoad<T>(load: () => Promise<T>, key: unknown): [Loaded<T>, () => void] {
  const [round, setRound] = useState(0);
  // The last answer, with the request it answers. Anything older than the current request
  // reads as "loading", so nothing has to be reset when a new request starts.
  const [answer, setAnswer] = useState<{ key: unknown; round: number; result: Loaded<T> } | null>(
    null,
  );

  useEffect(() => {
    let current = true;
    const settle = (result: Loaded<T>) => current && setAnswer({ key, round, result });
    load().then(
      (value) => settle({ state: "ready", value }),
      (thrown: unknown) =>
        settle({
          state: "failed",
          message: messageOf(thrown),
          kind: (thrown as { kind?: string }).kind ?? null,
        }),
    );
    return () => {
      current = false;
    };
    // `load` is a fresh closure on every render; `key` and `round` say when to run again.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, round]);

  const reload = useCallback(() => setRound((n) => n + 1), []);
  const fresh = answer !== null && Object.is(answer.key, key) && answer.round === round;
  return [fresh ? answer.result : { state: "loading" }, reload];
}
