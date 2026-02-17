import { invoke } from "@tauri-apps/api/core";

type JsonValue = string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue };

function maskSecretValue(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) return "";
  if (trimmed.length <= 6) return "*".repeat(trimmed.length);
  return `${trimmed.slice(0, 3)}***${trimmed.slice(-2)} (len=${trimmed.length})`;
}

function sanitize(value: unknown): JsonValue {
  if (value == null) return null;
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return value;
  if (Array.isArray(value)) return value.map((v) => sanitize(v));
  if (typeof value === "object") {
    const output: Record<string, JsonValue> = {};
    for (const [key, raw] of Object.entries(value as Record<string, unknown>)) {
      const lowered = key.toLowerCase();
      const isSensitiveKeyName =
        lowered === "apikey" ||
        lowered.endsWith("_apikey") ||
        lowered.endsWith("api_key") ||
        lowered === "key" ||
        lowered.endsWith("_key") ||
        lowered.endsWith("secret") ||
        lowered.endsWith("token") ||
        lowered.endsWith("password");

      if (isSensitiveKeyName) {
        output[key] = typeof raw === "string" ? maskSecretValue(raw) : "[masked]";
      } else {
        output[key] = sanitize(raw);
      }
    }
    return output;
  }
  return String(value);
}

export function buildSettingsConsistencySnapshot(settings: {
  sttProvider: string;
  sttModel: string;
  enhancementEnabled: boolean;
  enhancementProvider: string;
  enhancementModel: string;
  previewBeforeInsert: boolean;
}): JsonValue {
  const sttLocal = settings.sttProvider === "vosk";
  const enhancementLocal =
    settings.enhancementProvider === "ollama" || settings.enhancementProvider === "lmstudio";

  return {
    stt: {
      provider: settings.sttProvider,
      isLocal: sttLocal,
      needsApiKey: !sttLocal,
      model: settings.sttModel || "(empty)",
      modelConfigured: sttLocal ? true : settings.sttModel.trim().length > 0,
    },
    enhancement: {
      enabled: settings.enhancementEnabled,
      provider: settings.enhancementProvider,
      isLocal: enhancementLocal,
      needsApiKey: settings.enhancementEnabled ? !enhancementLocal : false,
      model: settings.enhancementModel || "(empty)",
      modelConfigured: settings.enhancementEnabled ? settings.enhancementModel.trim().length > 0 : true,
    },
    previewBeforeInsert: settings.previewBeforeInsert,
  };
}

export async function debugUiEvent(event: string, payload: unknown): Promise<void> {
  try {
    const raw = localStorage.getItem("voxlore-settings");
    if (raw) {
      const parsed = JSON.parse(raw) as { state?: { debugLoggingEnabled?: boolean } };
      if (parsed.state?.debugLoggingEnabled === false) {
        return;
      }
    }
  } catch {
    // ignore parse/storage errors
  }

  const safePayload = sanitize(payload);
  const jsonPayload = JSON.stringify(safePayload);
  console.info(`[ui-debug] ${event}`, safePayload);
  try {
    await invoke("debug_ui_event", { event, payload: jsonPayload });
  } catch {
    // 在未連接 Tauri（如純前端預覽）時忽略
  }
}
