export const DEFAULT_OPENROUTER_STT_MODEL = "google/gemini-3-flash-preview";

export const STT_PROVIDERS = [
  { value: "vosk", label: "Vosk (Local)" },
  { value: "openrouter", label: "OpenRouter" },
  { value: "custom_openai_compatible", label: "Custom OpenAI-Compatible" },
  { value: "openai_transcribe", label: "OpenAI Transcribe" },
  { value: "elevenlabs", label: "ElevenLabs" },
  { value: "openai", label: "OpenAI" },
  { value: "mistral", label: "Mistral" },
] as const;

export const ENHANCEMENT_PROVIDERS = [
  { value: "openrouter", label: "OpenRouter" },
  { value: "custom_openai_compatible", label: "Custom OpenAI-Compatible" },
  { value: "together", label: "Together" },
  { value: "groq", label: "Groq" },
  { value: "openai", label: "OpenAI" },
  { value: "ollama", label: "Ollama (Local)" },
  { value: "lmstudio", label: "LM Studio (Local)" },
] as const;

export const FLOATING_POSITIONS = [
  { value: "top-right", label: "Top Right" },
  { value: "bottom-right", label: "Bottom Right" },
  { value: "top-left", label: "Top Left" },
  { value: "bottom-left", label: "Bottom Left" },
] as const;

export const UI_LANGUAGES = [
  { value: "en", label: "English" },
  { value: "zh-TW", label: "繁體中文" },
  { value: "zh-CN", label: "简体中文" },
  { value: "ja", label: "日本語" },
] as const;

export const DEFAULT_SHORTCUTS = {
  pushToTalk: "Option+Space",
  toggleMode: "Option+Shift+Space",
} as const;
