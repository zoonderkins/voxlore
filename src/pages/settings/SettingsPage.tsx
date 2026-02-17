import { VoiceProviderSection } from "./VoiceProviderSection";
import { EnhancementSection } from "./EnhancementSection";
import { FloatingWindowSection } from "./FloatingWindowSection";
import { ShortcutsSection } from "./ShortcutsSection";
import { RecordingSection } from "./RecordingSection";
import { LanguageSection } from "./LanguageSection";
import { useTranslation } from "react-i18next";

export function SettingsPage() {
  const { t } = useTranslation();
  return (
    <div className="flex h-full flex-col bg-bg-primary">
      {/* Header */}
      <div className="flex items-center gap-3 border-b border-border px-6 py-4">
        <div className="flex h-10 w-10 items-center justify-center rounded-full bg-accent/20 text-lg">
          🎙
        </div>
        <div>
          <h1 className="text-lg font-bold text-text-primary">Voxlore</h1>
          <p className="text-xs text-success">● {t("common.ready")}</p>
        </div>
      </div>

      {/* Scrollable content */}
      <div className="flex-1 overflow-y-auto px-6 py-4">
        <div className="flex flex-col gap-8">
          <VoiceProviderSection />
          <EnhancementSection />
          <FloatingWindowSection />
          <ShortcutsSection />
          <RecordingSection />
          <LanguageSection />
        </div>

        {/* Footer */}
        <div className="mt-8 mb-4 text-center text-xs text-text-muted">
          <div>Voxlore v{__APP_VERSION__} — Open Source, Privacy First</div>
          <a
            href="https://github.com/zoonderkins/voxlore"
            target="_blank"
            rel="noreferrer"
            className="mt-1 inline-block text-accent hover:underline"
          >
            github.com/zoonderkins/voxlore
          </a>
        </div>
      </div>
    </div>
  );
}

declare const __APP_VERSION__: string;
