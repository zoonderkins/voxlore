interface ProgressBarProps {
  value: number; // 0-100
  label?: string;
  showPercent?: boolean;
}

export function ProgressBar({ value, label, showPercent = true }: ProgressBarProps) {
  const clampedValue = Math.max(0, Math.min(100, value));

  return (
    <div className="flex flex-col gap-1.5">
      {(label || showPercent) && (
        <div className="flex items-center justify-between text-xs">
          {label && <span className="text-text-secondary">{label}</span>}
          {showPercent && <span className="text-text-muted">{Math.round(clampedValue)}%</span>}
        </div>
      )}
      <div className="h-2 w-full overflow-hidden rounded-full bg-bg-hover">
        <div
          className="h-full rounded-full bg-accent"
          style={{ width: `${clampedValue}%`, transition: "width 150ms linear" }}
        />
      </div>
    </div>
  );
}
