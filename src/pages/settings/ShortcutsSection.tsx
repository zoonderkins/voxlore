import { ShortcutDisplay } from "../../components/ShortcutDisplay";
import { Card } from "../../components/ui";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { useTranslation } from "react-i18next";

export function ShortcutsSection() {
  const { t } = useTranslation();
  const { pushToTalkShortcut, toggleModeShortcut } = useSettingsStore();

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> {t("settings.shortcuts")}
      </h3>

      <Card padding="none" className="divide-y divide-border">
        <ShortcutDisplay
          shortcut={pushToTalkShortcut}
          title={t("settings.pushToTalk")}
          description={t("settings.pushToTalkDesc")}
        />
        <ShortcutDisplay
          shortcut={toggleModeShortcut}
          title={t("settings.toggleMode")}
          description={t("settings.toggleModeDesc")}
        />
      </Card>
    </section>
  );
}
