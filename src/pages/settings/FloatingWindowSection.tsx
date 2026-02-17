import { Toggle } from "../../components/ui";
import { ProviderSelector } from "../../components/ProviderSelector";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { FLOATING_POSITIONS } from "../../lib/constants";
import { hideFloatingWidget } from "../../lib/tauri";

export function FloatingWindowSection() {
  const { floatingWindowEnabled, floatingWindowPosition, previewBeforeInsert, updateSettings } =
    useSettingsStore();

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> Floating Window
      </h3>

      <Toggle
        label="Show floating window"
        description="Display status and progress in a floating panel"
        checked={floatingWindowEnabled}
        onChange={(v) => {
          updateSettings({ floatingWindowEnabled: v });
          if (!v) {
            hideFloatingWidget().catch(() => {});
          }
        }}
      />

      {floatingWindowEnabled && (
        <ProviderSelector
          label="Position"
          options={[...FLOATING_POSITIONS]}
          value={floatingWindowPosition}
          onChange={(v) =>
            updateSettings({ floatingWindowPosition: v as typeof floatingWindowPosition })
          }
        />
      )}

      <Toggle
        label="Preview before inserting"
        description="Review transcription and click Apply to insert"
        checked={previewBeforeInsert}
        onChange={(v) => updateSettings({ previewBeforeInsert: v })}
      />
    </section>
  );
}
