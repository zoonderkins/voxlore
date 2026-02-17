import { Card } from "../../../components/ui";
import { ShortcutDisplay } from "../../../components/ShortcutDisplay";
import { useSettingsStore } from "../../../stores/useSettingsStore";

export function HowToUseStep() {
  const { pushToTalkShortcut, toggleModeShortcut } = useSettingsStore();

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-2xl font-bold text-text-primary">How to Use</h2>
        <p className="mt-1 text-text-secondary">Two shortcuts to control voice input.</p>
      </div>

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

      <Card padding="md" className="flex items-center gap-3 bg-accent-muted border-accent/30">
        <span className="text-lg">A|</span>
        <span className="text-sm text-text-primary">
          Text is inserted at your cursor position in any app.
        </span>
      </Card>
    </div>
  );
}
