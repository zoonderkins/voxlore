import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { buildSettingsConsistencySnapshot, debugUiEvent } from "../lib/debug";
import { DEFAULT_OPENROUTER_STT_MODEL } from "../lib/constants";

export interface SettingsState {
  // Theme
  themeMode: "dark" | "light";

  // Voice Provider
  sttProvider:
    | "vosk"
    | "elevenlabs"
    | "openai"
    | "openai_transcribe"
    | "openrouter"
    | "custom_openai_compatible"
    | "mistral";
  sttModel: string;
  sttBaseUrl: string;
  sttLanguage: string;

  // Enhancement
  enhancementEnabled: boolean;
  enhancementProvider: string;
  enhancementModel: string;
  enhancementBaseUrl: string;

  // Floating Window
  floatingWindowEnabled: boolean;
  floatingWindowPosition: "top-right" | "bottom-right" | "top-left" | "bottom-left";
  previewBeforeInsert: boolean;

  // Shortcuts
  inputMode: "push-to-talk" | "toggle";
  pushToTalkShortcut: string;
  toggleModeShortcut: string;

  // Recording output
  outputDirectory: string;
  cloudTimeoutSecs: number;
  debugLoggingEnabled: boolean;
  rightClickDevtools: boolean;

  // Language
  uiLanguage: "en" | "zh-TW" | "zh-CN";

  // Setup
  setupCompleted: boolean;

  // Actions
  updateSettings: (partial: Partial<Omit<SettingsState, "updateSettings">>) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      // Defaults
      themeMode: "dark",
      sttProvider: "openrouter",
      sttModel: DEFAULT_OPENROUTER_STT_MODEL,
      sttBaseUrl: "",
      sttLanguage: "en",
      enhancementEnabled: false,
      enhancementProvider: "openrouter",
      enhancementModel: "google/gemini-3-flash-preview",
      enhancementBaseUrl: "",
      floatingWindowEnabled: false,
      floatingWindowPosition: "bottom-right",
      previewBeforeInsert: false,
      inputMode: "push-to-talk",
      pushToTalkShortcut: "Option+Space",
      toggleModeShortcut: "Option+Shift+Space",
      outputDirectory: "",
      cloudTimeoutSecs: 45,
      debugLoggingEnabled: true,
      rightClickDevtools: false,
      uiLanguage: "en",
      setupCompleted: false,

      updateSettings: (partial) =>
        set((current) => {
          const changed = Object.entries(partial).reduce<Record<string, unknown>>((acc, [k, v]) => {
            if ((current as unknown as Record<string, unknown>)[k] !== v) {
              acc[k] = v;
            }
            return acc;
          }, {});

          if (Object.keys(changed).length > 0) {
            const snapshot = buildSettingsConsistencySnapshot({
              sttProvider: (partial.sttProvider ?? current.sttProvider) as string,
              sttModel: (partial.sttModel ?? current.sttModel) as string,
              enhancementEnabled: Boolean(partial.enhancementEnabled ?? current.enhancementEnabled),
              enhancementProvider: (partial.enhancementProvider ?? current.enhancementProvider) as string,
              enhancementModel: (partial.enhancementModel ?? current.enhancementModel) as string,
              previewBeforeInsert: Boolean(partial.previewBeforeInsert ?? current.previewBeforeInsert),
            });
            void debugUiEvent("settings/update", { changed, snapshot });
          }

          return partial;
        }),
    }),
    {
      name: "voxlore-settings",
      storage: createJSONStorage(() => localStorage),
      onRehydrateStorage: () => (state) => {
        if (!state) return;
        if (state.sttProvider === "openrouter" && state.sttModel.trim().length === 0) {
          state.updateSettings({ sttModel: DEFAULT_OPENROUTER_STT_MODEL });
        }
      },
    },
  ),
);
