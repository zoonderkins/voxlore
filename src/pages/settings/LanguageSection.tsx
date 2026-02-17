import { Select } from "../../components/ui";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { UI_LANGUAGES } from "../../lib/constants";
import { useTranslation } from "react-i18next";

export function LanguageSection() {
  const { t } = useTranslation();
  const { uiLanguage, updateSettings } = useSettingsStore();

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> {t("settings.language")}
      </h3>

      <Select
        label={t("settings.interfaceLanguage")}
        options={[...UI_LANGUAGES]}
        value={uiLanguage}
        onChange={(v) => updateSettings({ uiLanguage: v as typeof uiLanguage })}
      />
    </section>
  );
}
