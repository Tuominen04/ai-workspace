import { useEffect, useRef } from "react";

type Props = {
  message: string;
  kind: string | null;
  /**
   * Makes the message dismissible: a click on it, Escape, or the next click anywhere else
   * clears it, so a refused action does not linger once the person moves on.
   */
  onDismiss?: () => void;
};

/** A failed engine call, said plainly, with a hint when the cause is known. */
export function Failure({ message, kind, onDismiss }: Props) {
  const box = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!onDismiss) return;
    const onPointer = (event: MouseEvent) => {
      if (!box.current?.contains(event.target as Node)) onDismiss();
    };
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onDismiss();
    };
    // A finished click (not a press) clears it, so the clicked button still gets its click even
    // when the message's going shifts the layout. Clicks count from the next tick, so the click
    // that caused the failure does not clear it.
    document.addEventListener("keydown", onKey);
    const timer = window.setTimeout(() => document.addEventListener("click", onPointer), 0);
    return () => {
      window.clearTimeout(timer);
      document.removeEventListener("click", onPointer);
      document.removeEventListener("keydown", onKey);
    };
  }, [onDismiss]);

  return (
    <div
      ref={box}
      className={onDismiss ? "failure dismissible" : "failure"}
      role="alert"
      onClick={onDismiss}
      title={onDismiss ? "Click to dismiss" : undefined}
    >
      <p>{message}</p>
      {kind === "ValidationGateBlocked" && (
        <p className="hint">Open the Validation tab to see what blocks it.</p>
      )}
      {onDismiss && (
        <button type="button" className="dismiss" aria-label="Dismiss" onClick={onDismiss}>
          ×
        </button>
      )}
    </div>
  );
}
