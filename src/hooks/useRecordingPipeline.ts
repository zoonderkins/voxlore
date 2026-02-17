import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useSettingsStore } from "../stores/useSettingsStore";
import { useRecordingStore } from "../stores/useRecordingStore";
import { useToastStore } from "../stores/useToastStore";
import type { RecordingResult } from "../lib/tauri";
import { enhanceText, insertTextAtCursor, showPreviewWindow, syncSettings } from "../lib/tauri";
import { buildSettingsConsistencySnapshot, debugUiEvent } from "../lib/debug";

/**
 * Hook that listens for recording pipeline events and handles
 * the preview vs direct-insert decision.
 * Also syncs relevant settings to Rust state.
 * Should be mounted once in the main window.
 */
export function useRecordingPipeline() {
  const previewBeforeInsert = useSettingsStore((s) => s.previewBeforeInsert);
  const floatingWindowEnabled = useSettingsStore((s) => s.floatingWindowEnabled);
  const floatingWindowPosition = useSettingsStore((s) => s.floatingWindowPosition);
  const sttLanguage = useSettingsStore((s) => s.sttLanguage);
  const sttProvider = useSettingsStore((s) => s.sttProvider);
  const sttModel = useSettingsStore((s) => s.sttModel);
  const sttBaseUrl = useSettingsStore((s) => s.sttBaseUrl);
  const cloudTimeoutSecs = useSettingsStore((s) => s.cloudTimeoutSecs);
  const debugLoggingEnabled = useSettingsStore((s) => s.debugLoggingEnabled);
  const enhancementEnabled = useSettingsStore((s) => s.enhancementEnabled);
  const enhancementProvider = useSettingsStore((s) => s.enhancementProvider);
  const enhancementModel = useSettingsStore((s) => s.enhancementModel);
  const enhancementBaseUrl = useSettingsStore((s) => s.enhancementBaseUrl);
  const uiLanguage = useSettingsStore((s) => s.uiLanguage);
  const { setStatus, setFinalText, reset } = useRecordingStore();
  const addToast = useToastStore((s) => s.addToast);

  const resolveEnhancementLanguage = (): string => {
    if (uiLanguage === "zh-TW" || sttLanguage === "zh") return "zh-TW";
    if (uiLanguage === "zh-CN") return "zh-CN";
    return "en";
  };

  const extractAiErrorToast = (message?: string): string | null => {
    if (!message) return null;
    const lower = message.toLowerCase();

    if (lower.includes("insufficient_quota") || lower.includes("429 too many requests")) {
      return "AI API 額度不足（429）。請檢查 billing / quota 後再試。";
    }
    if (lower.includes("invalid_api_key") || lower.includes("401")) {
      return "AI API 金鑰無效或已失效（401）。請更新 API Key。";
    }
    if (lower.includes("no api key configured for")) {
      const provider = message.match(/No API key configured for ([a-zA-Z0-9_\\-]+)/)?.[1];
      return provider
        ? `尚未設定 ${provider} API Key。請先到 Settings 儲存。`
        : "尚未設定 AI API Key。請先到 Settings 儲存。";
    }
    if (lower.includes("timeout") || lower.includes("timed out")) {
      return "AI 服務逾時。請檢查網路或稍後重試。";
    }
    if (lower.includes("network") || lower.includes("dns") || lower.includes("connection")) {
      return "無法連線 AI 服務。請檢查網路連線。";
    }
    if (lower.includes("api error") || lower.includes("transcription failed")) {
      return message.replace(/^Transcription failed:\\s*/, "").slice(0, 220);
    }

    return null;
  };

  // Sync settings to Rust whenever they change
  useEffect(() => {
      syncSettings({
        widgetPosition: floatingWindowPosition,
        floatingWindowEnabled,
        sttLanguage,
        sttProvider,
        sttModel,
        sttBaseUrl,
        cloudTimeoutSecs,
        debugLoggingEnabled,
      })
        .then(() =>
          debugUiEvent("settings/sync_to_rust", {
            widgetPosition: floatingWindowPosition,
            floatingWindowEnabled,
            sttLanguage,
            sttProvider,
            sttModel,
            sttBaseUrl,
            cloudTimeoutSecs,
            debugLoggingEnabled,
          }),
        )
        .catch(() => {});
  }, [
    cloudTimeoutSecs,
    debugLoggingEnabled,
    floatingWindowEnabled,
    floatingWindowPosition,
    sttLanguage,
    sttBaseUrl,
    sttModel,
    sttProvider,
  ]);

  // Listen for recording:status events
  useEffect(() => {
    const unlisten = listen<{ status: string; message?: string }>(
      "recording:status",
      (event) => {
        const { status, message } = event.payload;
        if (status === "recording") {
          reset();
          setStatus("recording");
        } else if (status === "processing") {
          setStatus("processing");
        } else if (status === "done") {
          setStatus("idle");
        } else if (status === "error") {
          useRecordingStore.getState().setError(message ?? "Unknown error");
          const toastMessage = extractAiErrorToast(message);
          if (toastMessage) {
            addToast(toastMessage, "error");
          }
        }
      },
    );
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [addToast, reset, setStatus]);

  // Listen for recording:result events (emitted after stop_recording succeeds)
  useEffect(() => {
    const unlisten = listen<RecordingResult>("recording:result", async (event) => {
      const { text } = event.payload;
      if (!text.trim()) return;
      void debugUiEvent(
        "recording/result",
        buildSettingsConsistencySnapshot({
          sttProvider,
          sttModel,
          enhancementEnabled: useSettingsStore.getState().enhancementEnabled,
          enhancementProvider: useSettingsStore.getState().enhancementProvider,
          enhancementModel: useSettingsStore.getState().enhancementModel,
          previewBeforeInsert,
        }),
      );

      let outputText = text;
      setFinalText(text);

      if (enhancementEnabled && outputText.trim()) {
        try {
          const language = resolveEnhancementLanguage();
          void debugUiEvent("enhancement/run", {
            provider: enhancementProvider,
            model: enhancementModel,
            language,
          });
          outputText = await enhanceText(
            outputText,
            enhancementProvider,
            enhancementModel,
            language,
            enhancementBaseUrl,
          );
          useRecordingStore.getState().setEnhancedText(outputText);
        } catch (e) {
          const message = String(e);
          void debugUiEvent("enhancement/error", { message });
          addToast("文字優化失敗，已使用原始轉錄結果。", "error");
        }
      }

      if (previewBeforeInsert) {
        setStatus("previewing");
        try {
          await showPreviewWindow(outputText);
        } catch (e) {
          console.error("Failed to show preview:", e);
        }
      } else {
        setStatus("inserting");
        try {
          const autoPasted = await insertTextAtCursor(outputText);
          if (!autoPasted) {
            useRecordingStore.getState().setClipboardOnly(true);
          }
        } catch (e) {
          console.error("Failed to insert text:", e);
        }
        setStatus("idle");
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [
    addToast,
    enhancementEnabled,
    enhancementModel,
    enhancementBaseUrl,
    enhancementProvider,
    previewBeforeInsert,
    setFinalText,
    setStatus,
    sttModel,
    sttProvider,
    sttLanguage,
    uiLanguage,
  ]);
}
