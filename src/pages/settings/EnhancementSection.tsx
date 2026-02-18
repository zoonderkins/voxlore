import { useCallback, useEffect, useState } from "react";
import { Card, Toggle, Select, Input, Button } from "../../components/ui";
import { ApiKeyInput } from "../../components/ApiKeyInput";
import { useSettingsStore } from "../../stores/useSettingsStore";
import { ENHANCEMENT_PROVIDERS } from "../../lib/constants";
import { debugUiEvent } from "../../lib/debug";
import { checkProviderHealth, type ProviderHealth } from "../../lib/tauri";
import { useToastStore } from "../../stores/useToastStore";
import { useTranslation } from "react-i18next";

const AUTO_HEALTH_CHECK_INTERVAL_MS = 60 * 60 * 1000;

const API_KEY_URLS: Record<string, string> = {
  openrouter: "https://openrouter.ai/keys",
  custom_openai_compatible: "",
  together: "https://api.together.xyz/settings/api-keys",
  groq: "https://console.groq.com/keys",
  openai: "https://platform.openai.com/api-keys",
};

const isLocalProvider = (p: string) => p === "ollama" || p === "lmstudio";
const isLikelyModelId = (value: string, allowPlainModel: boolean) => {
  const trimmed = value.trim();
  if (!trimmed) return false;
  if (/^[^/\s]+\/[^/\s].+/.test(trimmed)) return true;
  if (!allowPlainModel) return false;
  // 允許 LiteLLM / OpenAI-compatible 常見簡寫模型名（例如 gemini-3-flash）
  return /^[A-Za-z0-9][A-Za-z0-9._:-]*$/.test(trimmed);
};

const OPENROUTER_MODELS = [
  { value: "", label: "-- Select a model --" },
  { value: "qwen/qwen-turbo", label: "Qwen Turbo — Cheapest ($0.05/M)" },
  { value: "qwen/qwen-plus", label: "Qwen Plus — Better quality ($0.15/M)" },
  {
    value: "google/gemini-3-flash-preview",
    label: "Gemini 3 Flash Preview — Fast & balanced",
  },
  { value: "meta-llama/llama-4-scout", label: "Llama 4 Scout — Free tier" },
  { value: "custom", label: "Custom model..." },
] as const;

export function EnhancementSection() {
  const { t } = useTranslation();
  const addToast = useToastStore((s) => s.addToast);
  const {
    enhancementEnabled,
    enhancementProvider,
    enhancementModel,
    enhancementBaseUrl,
    updateSettings,
  } =
    useSettingsStore();
  const [health, setHealth] = useState<ProviderHealth | null>(null);
  const [healthLoading, setHealthLoading] = useState(false);

  const showModelSelector = enhancementProvider === "openrouter";
  const isRecommendedOpenRouterModel = OPENROUTER_MODELS.some(
    (m) => m.value === enhancementModel && m.value !== "" && m.value !== "custom",
  );
  const isCustomModel = !showModelSelector || !isRecommendedOpenRouterModel;
  const hasCustomEndpoint = enhancementBaseUrl.trim().length > 0;
  const needsCustomValidation =
    enhancementEnabled && !isLocalProvider(enhancementProvider) && (isCustomModel || !showModelSelector);
  const customModelValid = isLikelyModelId(enhancementModel, hasCustomEndpoint);
  const localizedModelOptions = OPENROUTER_MODELS.map((option) => {
    if (option.value === "") {
      return { ...option, label: t("settings.modelSelectPlaceholder") };
    }
    if (option.value === "custom") {
      return { ...option, label: t("settings.customModel") };
    }
    return option;
  });

  const runHealthCheck = useCallback(async (isManual = false) => {
    if (!enhancementEnabled) {
      setHealth(null);
      return;
    }
    if (isManual) {
      void debugUiEvent("enhancement/test_connection_click", {
        provider: enhancementProvider,
        model: enhancementModel,
        endpointConfigured: Boolean(enhancementBaseUrl.trim()),
      });
    }
    setHealthLoading(true);
    try {
      const result = await checkProviderHealth(
        "enhancement",
        enhancementProvider,
        enhancementModel,
        enhancementBaseUrl,
      );
      setHealth(result);
      if (isManual) {
        addToast(
          result.ok
            ? `文字增強連線測試成功（${result.status}）`
            : `文字增強連線測試失敗（${result.status}）`,
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
        addToast(`文字增強連線測試失敗（${String(e)}）`, "error");
      }
    } finally {
      setHealthLoading(false);
    }
  }, [
    addToast,
    enhancementBaseUrl,
    enhancementEnabled,
    enhancementModel,
    enhancementProvider,
  ]);

  useEffect(() => {
    if (!enhancementEnabled) {
      setHealth(null);
      setHealthLoading(false);
      return;
    }
    void runHealthCheck();
    const timer = setInterval(() => {
      void runHealthCheck();
    }, AUTO_HEALTH_CHECK_INTERVAL_MS);
    return () => {
      clearInterval(timer);
    };
  }, [enhancementEnabled, runHealthCheck]);

  return (
    <section className="flex flex-col gap-4">
      <h3 className="text-sm font-semibold text-text-primary flex items-center gap-2">
        <span className="text-accent">●</span> {t("settings.enhancement")}
      </h3>

      <Toggle
        label={t("settings.enableEnhancement")}
        description={t("settings.enableEnhancementDesc")}
        checked={enhancementEnabled}
        onChange={(v) => updateSettings({ enhancementEnabled: v })}
      />
      {enhancementEnabled && (
        <div className="text-xs">
          <span className={health?.ok ? "text-success" : "text-error"}>
            {healthLoading
              ? `● ${t("settings.health.checking")}`
              : health?.ok
                ? `● ${t("settings.health.healthy")}`
                : `● ${t("settings.health.unhealthy")}`}
          </span>
          <span className="ml-2 text-text-muted">
            {health?.status ?? t("settings.health.initializing")}
          </span>
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
      )}

      {enhancementEnabled && (
        <>
          <Select
            label={t("settings.provider")}
            options={[...ENHANCEMENT_PROVIDERS]}
            value={enhancementProvider}
            onChange={(v) => {
              void debugUiEvent("enhancement/provider_change", {
                from: enhancementProvider,
                to: v,
              });
              updateSettings({ enhancementProvider: v });
            }}
          />

          {!isLocalProvider(enhancementProvider) && (
            <Card padding="md">
              <ApiKeyInput
                provider={enhancementProvider}
                getKeyUrl={API_KEY_URLS[enhancementProvider]}
              />
            </Card>
          )}

          {showModelSelector && (
            <Select
              label={t("settings.models")}
              options={localizedModelOptions}
              value={isCustomModel ? "custom" : enhancementModel}
              onChange={(v) => {
                if (v === "custom") {
                  void debugUiEvent("enhancement/model_selector_custom", {
                    provider: enhancementProvider,
                  });
                  updateSettings({ enhancementModel: "" });
                  return;
                }
                if (v !== "") {
                  void debugUiEvent("enhancement/model_selector_pick", {
                    provider: enhancementProvider,
                    model: v,
                  });
                  updateSettings({ enhancementModel: v });
                }
              }}
            />
          )}

          {(isCustomModel || !showModelSelector) && (
            <Input
              label={t("settings.model")}
              placeholder={
                showModelSelector
                  ? t("settings.modelPlaceholderEnhancementSelector")
                  : t("settings.modelPlaceholderEnhancement")
              }
              value={enhancementModel}
              onChange={(e) => updateSettings({ enhancementModel: e.target.value })}
            />
          )}

          {!isLocalProvider(enhancementProvider) && (
            <Input
              label={
                hasCustomEndpoint
                  ? t("settings.customProviderEndpoint")
                  : t("settings.openAiCompatibleEndpoint")
              }
              placeholder={t("settings.endpointPlaceholder")}
              value={enhancementBaseUrl}
              onChange={(e) => updateSettings({ enhancementBaseUrl: e.target.value })}
            />
          )}

          {needsCustomValidation && (
            <p className={`text-xs ${customModelValid ? "text-success" : "text-error"}`}>
              {customModelValid
                ? t("settings.modelValidation.valid")
                : hasCustomEndpoint
                  ? t("settings.modelValidation.customEndpoint")
                  : t("settings.modelValidation.default")}
            </p>
          )}

          {isLocalProvider(enhancementProvider) && (
            <p className="text-xs text-text-muted">
              {t("settings.localProviderRunning", {
                provider: enhancementProvider === "ollama" ? "Ollama" : "LM Studio",
              })}
            </p>
          )}
        </>
      )}
    </section>
  );
}
