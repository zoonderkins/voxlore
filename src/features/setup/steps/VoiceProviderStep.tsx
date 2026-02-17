import { Card, Select, Input } from "../../../components/ui";
import { ProviderSelector } from "../../../components/ProviderSelector";
import { ApiKeyInput } from "../../../components/ApiKeyInput";
import { ModelManager } from "../../../components/ModelManager";
import { useSettingsStore } from "../../../stores/useSettingsStore";
import { DEFAULT_OPENROUTER_STT_MODEL, STT_PROVIDERS } from "../../../lib/constants";
import { debugUiEvent } from "../../../lib/debug";

const API_KEY_URLS: Record<string, string> = {
  elevenlabs: "https://elevenlabs.io/app/settings/api-keys",
  openai: "https://platform.openai.com/api-keys",
  openai_transcribe: "https://platform.openai.com/api-keys",
  mistral: "https://console.mistral.ai/api-keys",
  openrouter: "https://openrouter.ai/keys",
  custom_openai_compatible: "",
};

const LANGUAGE_OPTIONS = [
  { value: "en", label: "English" },
  { value: "zh", label: "Chinese (Mandarin)" },
  { value: "ja", label: "Japanese" },
  { value: "ko", label: "Korean" },
];

const UI_LANGUAGE_OPTIONS = [
  { value: "en", label: "English" },
  { value: "zh-TW", label: "繁體中文（台灣）" },
  { value: "zh-CN", label: "简体中文（中国）" },
];

const VOICE_MODEL_OPTIONS: Record<string, { value: string; label: string }[]> = {
  openrouter: [
    { value: "", label: "-- Select a model --" },
    { value: "google/gemini-3-flash-preview", label: "google/gemini-3-flash-preview" },
    { value: "openai/gpt-audio", label: "openai/gpt-audio" },
    { value: "openai/gpt-audio-mini", label: "openai/gpt-audio-mini" },
    { value: "mistralai/voxtral-small-24b-2507", label: "mistralai/voxtral-small-24b-2507" },
    { value: "meta-llama/llama-4-scout", label: "meta-llama/llama-4-scout" },
    { value: "custom", label: "Custom model..." },
  ],
  openai_transcribe: [
    { value: "", label: "-- Select a model --" },
    { value: "gpt-4o-mini-transcribe", label: "gpt-4o-mini-transcribe" },
    { value: "custom", label: "Custom model..." },
  ],
};

export function VoiceProviderStep() {
  const { uiLanguage, sttProvider, sttLanguage, sttModel, sttBaseUrl, updateSettings } = useSettingsStore();
  const isCloud = sttProvider !== "vosk";
  const modelOptions = VOICE_MODEL_OPTIONS[sttProvider];
  const hasModelSelector = Boolean(modelOptions);
  const matched = modelOptions?.some((o) => o.value === sttModel && o.value !== "custom");
  const selectedValue = hasModelSelector ? (matched ? sttModel : "custom") : sttModel;

  return (
    <div className="flex flex-col gap-6">
      <div>
        <h2 className="text-2xl font-bold text-text-primary">Welcome to Voxlore</h2>
        <p className="mt-1 text-text-secondary">
          Voice-to-text that types where your cursor is. Let's get you set up.
        </p>
      </div>

      <Card padding="md" className="flex flex-col gap-4">
        <ProviderSelector
          label="Voice Provider"
          options={[...STT_PROVIDERS]}
        value={sttProvider}
        onChange={(v) => {
          void debugUiEvent("setup/voice_provider_change", { from: sttProvider, to: v });
          const nextProvider = v as typeof sttProvider;
          updateSettings({
            sttProvider: nextProvider,
            sttModel:
              nextProvider === "openrouter"
                ? sttModel || DEFAULT_OPENROUTER_STT_MODEL
                : sttModel,
          });
        }}
      />

        {sttProvider === "vosk" && <ModelManager />}

        {isCloud && (
          <>
            <ApiKeyInput provider={sttProvider} getKeyUrl={API_KEY_URLS[sttProvider]} />
            {hasModelSelector && (
              <Select
                label="Models"
                options={modelOptions ?? []}
                value={selectedValue}
                onChange={(v) => {
                  if (v === "custom") {
                    updateSettings({ sttModel: "" });
                    return;
                  }
                  if (v !== "") {
                    updateSettings({ sttModel: v });
                  }
                }}
              />
            )}
            <Input
              label="Model (optional)"
              placeholder={
                sttProvider === "openai_transcribe"
                  ? "e.g. gpt-4o-mini-transcribe"
                  : sttProvider === "openrouter"
                    ? `e.g. ${DEFAULT_OPENROUTER_STT_MODEL}`
                    : sttProvider === "custom_openai_compatible"
                      ? "e.g. gemini-3-flash"
                    : "Leave empty for provider default"
              }
              value={sttModel}
              onChange={(e) => updateSettings({ sttModel: e.target.value })}
            />
            {(sttProvider === "openrouter"
              || sttProvider === "openai"
              || sttProvider === "openai_transcribe"
              || sttProvider === "custom_openai_compatible") && (
              <Input
                label="OpenAI Compatible Endpoint (optional)"
                placeholder="e.g. https://your-openai-compatible-endpoint/v1"
                value={sttBaseUrl}
                onChange={(e) => updateSettings({ sttBaseUrl: e.target.value })}
              />
            )}
          </>
        )}

        <Select
          label="UI Language"
          options={UI_LANGUAGE_OPTIONS}
          value={uiLanguage}
          onChange={(v) => updateSettings({ uiLanguage: v as typeof uiLanguage })}
        />

        <Select
          label="Speech Language (STT)"
          options={LANGUAGE_OPTIONS}
          value={sttLanguage}
          onChange={(v) => updateSettings({ sttLanguage: v })}
        />
      </Card>
    </div>
  );
}
