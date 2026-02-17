import { useEffect, useState } from "react";
import { Card, Toggle } from "../../components/ui";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { getRecordingsDir } from "../../lib/tauri";
import { useTranslation } from "react-i18next";

export function RecordingSection() {
  const { t } = useTranslation();
  const {
    outputDirectory,
    cloudTimeoutSecs,
    debugLoggingEnabled,
    rightClickDevtools,
    updateSettings,
  } = useSettingsStore();
  const [defaultDir, setDefaultDir] = useState("");
  const [draft, setDraft] = useState(outputDirectory);
  const isDirty = draft !== outputDirectory;

  useEffect(() => {
    getRecordingsDir().then(setDefaultDir).catch(console.error);
  }, []);

  // Sync draft when store changes externally
  useEffect(() => {
    setDraft(outputDirectory);
  }, [outputDirectory]);

  const displayPath = outputDirectory || defaultDir || "~/Documents/Voxlore/recordings";

  const handleSave = () => {
    updateSettings({ outputDirectory: draft });
  };

  const handleReset = () => {
    setDraft("");
    updateSettings({ outputDirectory: "" });
  };

  const handleCancel = () => {
    setDraft(outputDirectory);
  };

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> {t("settings.recordingOutput")}
      </h3>

      <Card padding="md" className="flex flex-col gap-3">
        <div className="flex items-center justify-between">
          <div className="flex flex-col gap-1">
            <span className="text-sm text-text-primary">{t("settings.saveRecordingsTo")}</span>
            <span className="text-xs text-text-muted font-mono truncate max-w-[280px]">
              {displayPath}
            </span>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <input
            type="text"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder={t("settings.recordingPathPlaceholder")}
            className="flex-1 rounded-lg border border-border bg-bg-primary px-3 py-1.5 text-xs text-text-primary placeholder:text-text-muted focus:border-accent focus:outline-none"
          />
        </div>

        {/* Action buttons: show Save/Cancel when editing, Reset when custom path is saved */}
        <div className="flex items-center gap-2">
          {isDirty && (
            <>
              <button
                onClick={handleSave}
                className="rounded-lg bg-accent px-3 py-1.5 text-xs font-medium text-white hover:bg-accent/90 transition-colors"
              >
                {t("common.save")}
              </button>
              <button
                onClick={handleCancel}
                className="rounded-lg border border-border px-3 py-1.5 text-xs text-text-muted hover:text-text-primary hover:border-accent transition-colors"
              >
                {t("common.cancel")}
              </button>
            </>
          )}
          {!isDirty && outputDirectory && (
            <button
              onClick={handleReset}
              className="rounded-lg border border-border px-3 py-1.5 text-xs text-text-muted hover:text-text-primary hover:border-accent transition-colors"
            >
              {t("settings.resetToDefault")}
            </button>
          )}
        </div>
      </Card>

      <Card padding="md" className="flex flex-col gap-3">
        <div className="flex flex-col gap-1">
          <span className="text-sm text-text-primary">{t("settings.cloudTimeoutSeconds")}</span>
          <span className="text-xs text-text-muted">
            {t("settings.cloudTimeoutDesc")}
          </span>
        </div>
        <input
          type="number"
          min={5}
          max={180}
          value={cloudTimeoutSecs}
          onChange={(e) => {
            const n = Number(e.target.value || 45);
            updateSettings({ cloudTimeoutSecs: Math.max(5, Math.min(180, Math.floor(n))) });
          }}
          className="w-28 rounded-lg border border-border bg-bg-primary px-3 py-1.5 text-xs text-text-primary focus:border-accent focus:outline-none"
        />
      </Card>

      <Card padding="md" className="flex flex-col gap-3">
        <Toggle
          label={t("settings.debugLogs")}
          description={t("settings.debugLogsDesc")}
          checked={debugLoggingEnabled}
          onChange={(v) => updateSettings({ debugLoggingEnabled: v })}
        />
        <Toggle
          label={t("settings.rightClickDevtools")}
          description={t("settings.rightClickDevtoolsDesc")}
          checked={rightClickDevtools}
          onChange={(v) => updateSettings({ rightClickDevtools: v })}
        />
      </Card>
    </section>
  );
}
