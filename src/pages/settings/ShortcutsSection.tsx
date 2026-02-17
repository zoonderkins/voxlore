import { ShortcutDisplay } from "../../components/ShortcutDisplay";
import { Card } from "../../components/ui";
import { useSettingsStore } from "../../stores/useSettingsStore";

export function ShortcutsSection() {
  const { pushToTalkShortcut, toggleModeShortcut } = useSettingsStore();

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> Shortcuts
      </h3>

      <Card padding="none" className="divide-y divide-border">
        <ShortcutDisplay
          shortcut={pushToTalkShortcut}
          title="Push-to-talk"
          description="Hold to record, release to transcribe and insert"
        />
        <ShortcutDisplay
          shortcut={toggleModeShortcut}
          title="Toggle mode"
          description="Press to start, press again to stop and insert"
        />
      </Card>
    </section>
  );
}
