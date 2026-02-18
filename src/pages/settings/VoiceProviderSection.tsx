import { useCallback, useEffect, useState } from "react";
import { Card, Select, Input, Button } from "../../components/ui";
import { ProviderSelector } from "../../components/ProviderSelector";
import { ApiKeyInput } from "../../components/ApiKeyInput";
import { ModelManager } from "../../components/ModelManager";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { DEFAULT_OPENROUTER_STT_MODEL, STT_PROVIDERS } from "../../lib/constants";
import { debugUiEvent } from "../../lib/debug";
import { checkProviderHealth, type ProviderHealth } from "../../lib/tauri";
import { useToastStore } from "../../stores/useToastStore";

const AUTO_HEALTH_CHECK_INTERVAL_MS = 60 * 60 * 1000;

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
  { value: "es", label: "Spanish" },
  { value: "fr", label: "French" },
  { value: "de", label: "German" },
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

export function VoiceProviderSection() {
  const { sttProvider, sttLanguage, sttModel, sttBaseUrl, updateSettings } = useSettingsStore();
  const addToast = useToastStore((s) => s.addToast);
  const isCloud = sttProvider !== "vosk";
  const modelOptions = VOICE_MODEL_OPTIONS[sttProvider];
  const hasModelSelector = Boolean(modelOptions);
  const matched = modelOptions?.some((o) => o.value === sttModel && o.value !== "custom");
  const selectedValue = hasModelSelector ? (matched ? sttModel : "custom") : sttModel;
  const hasCustomEndpoint = sttBaseUrl.trim().length > 0;
  const [health, setHealth] = useState<ProviderHealth | null>(null);
  const [healthLoading, setHealthLoading] = useState(false);

  const runHealthCheck = useCallback(async (isManual = false) => {
    if (isManual) {
      void debugUiEvent("voice/test_connection_click", {
        provider: sttProvider,
        model: sttModel,
        endpointConfigured: Boolean(sttBaseUrl.trim()),
      });
    }
    setHealthLoading(true);
    try {
      const result = await checkProviderHealth("voice", sttProvider, sttModel, sttBaseUrl);
      setHealth(result);
      if (isManual) {
        addToast(
          result.ok
            ? `語音連線測試成功（${result.status}）`
            : `語音連線測試失敗（${result.status}）`,
          result.ok ? "success" : "error",
        );
      }
    } catch (e) {
      setHealth({
        ok: false,
        hasKey: false,
        latencyMs: null,
        status: String(e),
      });
      if (isManual) {
        addToast(`語音連線測試失敗（${String(e)}）`, "error");
      }
    } finally {
      setHealthLoading(false);
    }
  }, [addToast, sttBaseUrl, sttModel, sttProvider]);

  useEffect(() => {
    void runHealthCheck();
    const timer = setInterval(() => {
      void runHealthCheck();
    }, AUTO_HEALTH_CHECK_INTERVAL_MS);
    return () => {
      clearInterval(timer);
    };
  }, [runHealthCheck]);

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> Voice Provider
      </h3>
      <div className="text-xs">
        <span className={health?.ok ? "text-success" : "text-error"}>
          {healthLoading ? "● checking..." : health?.ok ? "● healthy" : "● unhealthy"}
        </span>
        <span className="ml-2 text-text-muted">{health?.status ?? "initializing..."}</span>
        <Button
          type="button"
          variant="secondary"
          size="sm"
          className="ml-3"
          disabled={healthLoading}
          onClick={() => {
            void runHealthCheck(true);
          }}
        >
          {healthLoading ? "測試中..." : "測試連線"}
        </Button>
      </div>

      <ProviderSelector
        label="Provider"
        options={[...STT_PROVIDERS]}
        value={sttProvider}
        onChange={(v) => {
          void debugUiEvent("voice/provider_change", { from: sttProvider, to: v });
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

      {sttProvider === "vosk" && (
        <Card padding="md">
          <ModelManager />
        </Card>
      )}

      {isCloud && (
        <Card padding="md" className="flex flex-col gap-4">
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
              label={
                hasCustomEndpoint
                  ? "Custom Provider Endpoint (OpenAI-compatible)"
                  : "OpenAI Compatible Endpoint (optional)"
              }
              placeholder="e.g. https://your-openai-compatible-endpoint/v1"
              value={sttBaseUrl}
              onChange={(e) => updateSettings({ sttBaseUrl: e.target.value })}
            />
          )}
        </Card>
      )}

      <Select
        label="Language"
        options={LANGUAGE_OPTIONS}
        value={sttLanguage}
        onChange={(v) => updateSettings({ sttLanguage: v })}
      />
    </section>
  );
}
