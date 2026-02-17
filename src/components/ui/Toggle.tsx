interface ToggleProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

export function Toggle({ label, description, checked, onChange, disabled = false }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange(!checked)}
      className={`flex w-full items-center justify-between rounded-lg border border-border bg-bg-card p-4 transition-colors duration-150 ${disabled ? "cursor-not-allowed opacity-50" : "cursor-pointer hover:bg-bg-hover"}`}
    >
      <div className="flex flex-col items-start gap-0.5">
        <span className="text-sm font-semibold text-text-primary">{label}</span>
        {description && <span className="text-xs text-text-muted">{description}</span>}
      </div>
      <div
        className={`relative h-6 w-11 rounded-full transition-colors duration-200 ${checked ? "bg-accent" : "bg-bg-hover"}`}
      >
        <div
          className={`absolute top-0.5 h-5 w-5 rounded-full bg-white shadow-sm transition-transform duration-200 ${checked ? "translate-x-5.5" : "translate-x-0.5"}`}
        />
      </div>
    </button>
  );
}
