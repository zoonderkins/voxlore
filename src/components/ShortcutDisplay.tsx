interface ShortcutDisplayProps {
  shortcut: string;
  title: string;
  description: string;
}

export function ShortcutDisplay({ shortcut, title, description }: ShortcutDisplayProps) {
  const keys = shortcut.split("+");
  return (
    <div className="flex items-center gap-4 p-3">
      <div className="flex items-center gap-1">
        {keys.map((key, i) => (
          <span key={i}>
            {i > 0 && <span className="text-text-muted mx-0.5">+</span>}
            <kbd className="inline-flex min-w-[2rem] items-center justify-center rounded-md border border-border bg-bg-input px-2 py-1 text-xs font-mono text-text-primary">
              {key.trim()}
            </kbd>
          </span>
        ))}
      </div>
      <div className="flex flex-col">
        <span className="text-sm font-semibold text-text-primary">{title}</span>
        <span className="text-xs text-text-muted">{description}</span>
      </div>
    </div>
  );
}
