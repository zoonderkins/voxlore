import { Toggle } from "../../components/ui";
import { ProviderSelector } from "../../components/ProviderSelector";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { hideFloatingWidget } from "../../lib/tauri";
import { useTranslation } from "react-i18next";

export function FloatingWindowSection() {
  const { t } = useTranslation();
  const { floatingWindowEnabled, floatingWindowPosition, previewBeforeInsert, updateSettings } =
    useSettingsStore();
  const floatingPositions = [
    { value: "top-right", label: t("settings.floatingPositions.topRight") },
    { value: "bottom-right", label: t("settings.floatingPositions.bottomRight") },
    { value: "top-left", label: t("settings.floatingPositions.topLeft") },
    { value: "bottom-left", label: t("settings.floatingPositions.bottomLeft") },
  ] as const;

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> {t("settings.floatingWindow")}
      </h3>

      <Toggle
        label={t("settings.showFloatingWindow")}
        description={t("settings.showFloatingWindowDesc")}
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
          label={t("settings.position")}
          options={[...floatingPositions]}
          value={floatingWindowPosition}
          onChange={(v) =>
            updateSettings({ floatingWindowPosition: v as typeof floatingWindowPosition })
          }
        />
      )}

      <Toggle
        label={t("settings.previewBeforeInsert")}
        description={t("settings.previewBeforeInsertDesc")}
        checked={previewBeforeInsert}
        onChange={(v) => updateSettings({ previewBeforeInsert: v })}
      />
    </section>
  );
}
