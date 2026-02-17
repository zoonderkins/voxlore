import { invoke } from "@tauri-apps/api/core";

// --- Types matching Rust structs ---

export interface AppInfo {
  name: string;
  version: string;
}

export interface SttResult {
  text: string;
  confidence: number | null;
  language_detected: string | null;
}

export type SttProvider =
  | "vosk"
  | "elevenlabs"
  | "openai"
  | "openai_transcribe"
  | "openrouter"
  | "custom_openai_compatible"
  | "mistral";

export type RecordingStatus = "idle" | "recording" | "processing" | "error";

export interface RecordingResult {
  text: string;
  audioPath: string | null;
  textPath: string | null;
  durationSecs: number;
}

export interface ProviderHealth {
  ok: boolean;
  hasKey: boolean;
  latencyMs: number | null;
  status: string;
}

// --- Typed invoke wrappers ---

export async function getAppInfo(): Promise<AppInfo> {
  return invoke<AppInfo>("get_app_info");
}

export async function greet(name: string): Promise<string> {
  return invoke<string>("greet", { name });
}

export async function startRecording(): Promise<void> {
  return invoke<void>("start_recording");
}

export async function stopRecording(outputDir?: string): Promise<RecordingResult> {
  return invoke<RecordingResult>("stop_recording", { outputDir: outputDir ?? null });
}

export async function getRecordingsDir(): Promise<string> {
  return invoke<string>("get_recordings_dir");
}

export async function saveApiKey(provider: string, key: string): Promise<void> {
  return invoke<void>("save_api_key", { provider, key });
}

export async function hasApiKey(provider: string): Promise<boolean> {
  return invoke<boolean>("has_api_key", { provider });
}

export async function deleteApiKey(provider: string): Promise<void> {
  return invoke<void>("delete_api_key", { provider });
}

export async function checkProviderHealth(
  section: "voice" | "enhancement",
  provider: string,
  model?: string,
  endpoint?: string,
): Promise<ProviderHealth> {
  return invoke<ProviderHealth>("check_provider_health", {
    section,
    provider,
    model: model ?? null,
    endpoint: endpoint ?? null,
  });
}

export async function syncSettings(settings: {
  widgetPosition?: string;
  floatingWindowEnabled?: boolean;
  sttLanguage?: string;
  sttProvider?: SttProvider;
  sttModel?: string;
  sttBaseUrl?: string;
  cloudTimeoutSecs?: number;
  debugLoggingEnabled?: boolean;
}): Promise<void> {
  return invoke<void>("sync_settings", {
    widgetPosition: settings.widgetPosition ?? null,
    floatingWindowEnabled: settings.floatingWindowEnabled ?? null,
    sttLanguage: settings.sttLanguage ?? null,
    sttProvider: settings.sttProvider ?? null,
    sttModel: settings.sttModel ?? null,
    sttBaseUrl: settings.sttBaseUrl ?? null,
    cloudTimeoutSecs: settings.cloudTimeoutSecs ?? null,
    debugLoggingEnabled: settings.debugLoggingEnabled ?? null,
  });
}

export async function enhanceText(
  text: string,
  provider: string,
  model: string,
  language?: string,
  endpoint?: string,
): Promise<string> {
  return invoke<string>("enhance_text", {
    text,
    provider,
    model,
    language: language ?? null,
    endpoint: endpoint ?? null,
  });
}

/** Returns `true` if auto-pasted, `false` if text is on clipboard only. */
export async function insertTextAtCursor(text: string): Promise<boolean> {
  return invoke<boolean>("insert_text_at_cursor", { text });
}

// --- Vosk Model Management ---

export interface VoskModel {
  id: string;
  name: string;
  language: string;
  size_mb: number;
  url: string;
  description: string;
}

export interface VoskModelStatus {
  loaded: boolean;
  modelId: string | null;
  modelPath: string | null;
}

export interface DownloadProgress {
  modelId: string;
  downloaded: number;
  total: number;
  percent: number;
  stage: "downloading" | "extracting" | "complete";
}

export async function listVoskModels(): Promise<VoskModel[]> {
  return invoke<VoskModel[]>("list_vosk_models");
}

export async function downloadVoskModel(modelId: string): Promise<string> {
  return invoke<string>("download_vosk_model", { modelId });
}

export async function loadVoskModel(modelId: string): Promise<VoskModelStatus> {
  return invoke<VoskModelStatus>("load_vosk_model", { modelId });
}

export async function unloadVoskModel(): Promise<VoskModelStatus> {
  return invoke<VoskModelStatus>("unload_vosk_model");
}

export async function getVoskStatus(): Promise<VoskModelStatus> {
  return invoke<VoskModelStatus>("get_vosk_status");
}

export async function listDownloadedVoskModels(): Promise<string[]> {
  return invoke<string[]>("list_downloaded_vosk_models");
}

export async function transcribeAudio(
  audioData: number[],
  provider: SttProvider,
  language: string,
  model?: string,
): Promise<SttResult> {
  return invoke<SttResult>("transcribe_audio", {
    audioData,
    provider,
    language,
    model: model ?? null,
  });
}

// --- Permissions ---

export interface PermissionStatus {
  microphone: string;
  accessibility: string;
}

export async function checkPermissions(): Promise<PermissionStatus> {
  return invoke<PermissionStatus>("check_permissions");
}

export async function requestMicrophonePermission(): Promise<boolean> {
  return invoke<boolean>("request_microphone_permission");
}

export async function requestAccessibilityPermission(): Promise<void> {
  return invoke<void>("request_accessibility_permission");
}

// --- Floating Widget ---

export async function showFloatingWidget(position?: string): Promise<void> {
  return invoke<void>("show_floating_widget", { position: position ?? null });
}

export async function hideFloatingWidget(): Promise<void> {
  return invoke<void>("hide_floating_widget");
}

// --- Preview Window ---

export async function showPreviewWindow(text: string): Promise<void> {
  return invoke<void>("show_preview_window", { text });
}

export async function getPreviewText(): Promise<string | null> {
  return invoke<string | null>("get_preview_text");
}

export async function closePreviewWindow(): Promise<void> {
  return invoke<void>("close_preview_window");
}

/** Returns `true` if auto-pasted, `false` if text is on clipboard only. */
export async function applyPreviewText(text: string): Promise<boolean> {
  return invoke<boolean>("apply_preview_text", { text });
}
