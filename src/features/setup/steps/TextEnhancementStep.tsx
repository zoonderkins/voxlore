import { Card, Toggle, Select, Input } from "../../../components/ui";
import { ApiKeyInput } from "../../../components/ApiKeyInput";
import { useSettingsStore } from "../../../stores/useSettingsStore";
import { ENHANCEMENT_PROVIDERS } from "../../../lib/constants";

const API_KEY_URLS: Record<string, string> = {
  openrouter: "https://openrouter.ai/keys",
  custom_openai_compatible: "",
  together: "https://api.together.xyz/settings/api-keys",
  groq: "https://console.groq.com/keys",
  openai: "https://platform.openai.com/api-keys",
};

const isLocalProvider = (p: string) => p === "ollama" || p === "lmstudio";

export function TextEnhancementStep() {
  const {
    enhancementEnabled,
    enhancementProvider,
    enhancementModel,
    enhancementBaseUrl,
    updateSettings,
  } =
    useSettingsStore();

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-2xl font-bold text-text-primary">Text Enhancement</h2>
        <p className="mt-1 text-text-secondary">
          An LLM can fix grammar, add punctuation, and clean up your transcription.
        </p>
      </div>

      <Toggle
        label="Enable enhancement"
        description="Optional — you can enable this later in Settings"
        checked={enhancementEnabled}
        onChange={(v) => updateSettings({ enhancementEnabled: v })}
      />

      {enhancementEnabled && (
        <Card padding="md" className="flex flex-col gap-4">
          <Select
            label="Provider"
            options={[...ENHANCEMENT_PROVIDERS]}
            value={enhancementProvider}
            onChange={(v) => updateSettings({ enhancementProvider: v })}
          />

          {!isLocalProvider(enhancementProvider) && (
            <ApiKeyInput
              provider={enhancementProvider}
              getKeyUrl={API_KEY_URLS[enhancementProvider]}
            />
          )}

          <Input
            label="Model"
            placeholder="e.g. google/gemini-3-flash-preview"
            value={enhancementModel}
            onChange={(e) => updateSettings({ enhancementModel: e.target.value })}
          />
          {!isLocalProvider(enhancementProvider) && (
            <Input
              label="OpenAI Compatible Endpoint (optional)"
              placeholder="e.g. https://your-openai-compatible-endpoint/v1"
              value={enhancementBaseUrl}
              onChange={(e) => updateSettings({ enhancementBaseUrl: e.target.value })}
            />
          )}
        </Card>
      )}
    </div>
  );
}
