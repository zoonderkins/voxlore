import { useSettingsStore } from "../stores/useSettingsStore";

export function ThemeToggle() {
  const themeMode = useSettingsStore((s) => s.themeMode);
  const updateSettings = useSettingsStore((s) => s.updateSettings);

  const isDark = themeMode === "dark";
  const nextMode = isDark ? "light" : "dark";

  return (
    <button
      type="button"
      aria-label={`Switch to ${nextMode} mode`}
      title={`Switch to ${nextMode} mode`}
      onClick={() => updateSettings({ themeMode: nextMode })}
      className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-border bg-bg-card text-base transition-colors duration-150 hover:bg-bg-hover"
    >
      {isDark ? "☀️" : "🌙"}
    </button>
  );
}
