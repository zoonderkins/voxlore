export function CompletionStep() {
  return (
    <div className="flex flex-col items-center justify-center gap-6 py-8">
      <div className="flex h-20 w-20 items-center justify-center rounded-full bg-success/20 text-4xl">
        ✓
      </div>

      <div className="text-center">
        <h2 className="text-2xl font-bold text-text-primary">You're all set!</h2>
        <p className="mt-2 text-text-secondary">
          Voxlore is ready to use. You can change any of these settings later.
        </p>
      </div>

      <div className="mt-4 w-full max-w-sm rounded-xl bg-bg-card p-4 text-center">
        <p className="text-sm text-text-secondary">
          Try it now — hold <kbd className="rounded border border-border bg-bg-input px-1.5 py-0.5 font-mono text-xs">Option</kbd> + <kbd className="rounded border border-border bg-bg-input px-1.5 py-0.5 font-mono text-xs">Space</kbd> and speak!
        </p>
      </div>
    </div>
  );
}
