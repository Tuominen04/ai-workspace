/** A failed engine call, said plainly, with a hint when the cause is known. */
export function Failure({ message, kind }: { message: string; kind: string | null }) {
  return (
    <div className="failure" role="alert">
      <p>{message}</p>
      {kind === "ValidationGateBlocked" && (
        <p className="hint">Open the Validation tab to see what blocks it.</p>
      )}
    </div>
  );
}
